import { useState, useCallback, useEffect, useRef } from 'react'
import { api } from '@/api/current'
import type {
  DeviceInfo,
  NetworkInfo,
  CellsResponse,
  QosInfo,
  SimInfo,
  SystemStatsResponse,
  AirplaneModeResponse,
  RoamingResponse,
  NetworkInterfaceInfo,
} from '@/api/types'
import { publicIpv6Addresses } from '@/utils/ip'

export const SPEED_HISTORY_MAX_POINTS = 30

/** ModemManager 通常不暴露 QCI；在数据连接开启时从 WWAN 网卡字节速率估算上下行（kbps，与旧 QosInfo 字段一致）。 */
function qosFromWwanInterface(stats: SystemStatsResponse, dataActive: boolean): QosInfo | null {
  if (!dataActive || !stats.network_speed?.interfaces?.length) return null
  const wwan = stats.network_speed.interfaces.find(
    (i) =>
      i.interface.startsWith('wwan') ||
      i.interface.startsWith('wwp') ||
      i.interface.toLowerCase().includes('mbim'),
  )
  if (!wwan) return null
  return {
    qci: 0,
    dl_speed: (wwan.rx_bytes_per_sec * 8) / 1000,
    ul_speed: (wwan.tx_bytes_per_sec * 8) / 1000,
    source: 'interface',
  }
}

export interface InterfaceSpeedHistory {
  rx: number[]
  tx: number[]
  totalRx: number
  totalTx: number
}

export interface ConnectivityResult {
  ipv4: { success: boolean; latency_ms?: number }
  ipv6: { success: boolean; latency_ms?: number }
}

export interface ConnectionAddresses {
  ipv4: string[]
  ipv6: string[]
}

function connectionAddressesFromInterfaces(interfaces: NetworkInterfaceInfo[]): ConnectionAddresses {
  const preferredWlan = interfaces.find(
    (iface) => iface.status !== 'down' && iface.name === 'wlan0' && iface.ip_addresses.length > 0,
  )
  const fallback = interfaces.find((iface) => iface.status !== 'down' && iface.ip_addresses.length > 0)
  const selected = preferredWlan ?? fallback

  if (!selected) {
    return { ipv4: [], ipv6: [] }
  }

  const wlanPublicIpv6 = publicIpv6Addresses(preferredWlan?.ip_addresses ?? [])
  const fallbackPublicIpv6 = publicIpv6Addresses(selected.ip_addresses)

  return {
    ipv4: selected.ip_addresses.filter((ip) => ip.ip_type === 'ipv4').map((ip) => ip.address),
    ipv6: wlanPublicIpv6.length ? wlanPublicIpv6 : fallbackPublicIpv6,
  }
}

export interface DashboardData {
  deviceInfo: DeviceInfo | null
  simInfo: SimInfo | null
  systemStats: SystemStatsResponse | null
  networkInfo: NetworkInfo | null
  dataStatus: boolean
  cellsInfo: CellsResponse | null
  qosInfo: QosInfo | null
  airplaneMode: AirplaneModeResponse | null
  connectivity: ConnectivityResult | null
  connectionAddresses: ConnectionAddresses
  speedHistory: Record<string, InterfaceSpeedHistory>
  roaming: RoamingResponse | null
}

export interface DashboardActions {
  toggleData: () => Promise<void>
  toggleAirplaneMode: () => Promise<void>
  toggleRoaming: () => Promise<void>
  loadData: () => Promise<void>
}

export function useDashboardData(refreshInterval: number, refreshKey: number) {
  const [initialLoading, setInitialLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null)
  const [simInfo, setSimInfo] = useState<SimInfo | null>(null)
  const [systemStats, setSystemStats] = useState<SystemStatsResponse | null>(null)
  const [networkInfo, setNetworkInfo] = useState<NetworkInfo | null>(null)
  const [dataStatus, setDataStatus] = useState(false)
  const [cellsInfo, setCellsInfo] = useState<CellsResponse | null>(null)
  const [qosInfo, setQosInfo] = useState<QosInfo | null>(null)
  const [airplaneMode, setAirplaneMode] = useState<AirplaneModeResponse | null>(null)
  const [connectivity, setConnectivity] = useState<ConnectivityResult | null>(null)
  const [connectionAddresses, setConnectionAddresses] = useState<ConnectionAddresses>({ ipv4: [], ipv6: [] })
  const [roaming, setRoaming] = useState<RoamingResponse | null>(null)
  const [speedHistory, setSpeedHistory] = useState<Record<string, InterfaceSpeedHistory>>({})
  const speedHistoryRef = useRef<Record<string, InterfaceSpeedHistory>>({})

  const updateSpeedHistory = useCallback((stats: SystemStatsResponse | null) => {
    if (!stats?.network_speed?.interfaces) return

    const nextHistory = { ...speedHistoryRef.current }

    for (const iface of stats.network_speed.interfaces) {
      const existing = nextHistory[iface.interface] || { rx: [], tx: [], totalRx: 0, totalTx: 0 }
      const rx = [...existing.rx, iface.rx_bytes_per_sec]
      const tx = [...existing.tx, iface.tx_bytes_per_sec]

      if (rx.length > SPEED_HISTORY_MAX_POINTS) {
        rx.shift()
        tx.shift()
      }

      nextHistory[iface.interface] = {
        rx,
        tx,
        totalRx: iface.total_rx_bytes,
        totalTx: iface.total_tx_bytes,
      }
    }

    speedHistoryRef.current = nextHistory
    setSpeedHistory(nextHistory)
  }, [])

  const loadData = useCallback(async () => {
    setError(null)
    const failures: string[] = []

    const requestOrNull = async <T,>(promise: Promise<T>, label: string): Promise<T | null> => {
      try {
        return await promise
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err)
        failures.push(`${label}: ${message}`)
        return null
      }
    }

    try {
      // 快速请求：决定 initialLoading，通常 <200ms 即可全部返回
      const fastPromise = Promise.all([
        requestOrNull(api.getDeviceInfo(), 'device'),
        requestOrNull(api.getSimInfo(), 'sim'),
        requestOrNull(api.getNetworkInfo(), 'network'),
        requestOrNull(api.getDataStatus(), 'data'),
        requestOrNull(api.getAirplaneMode(), 'airplane-mode'),
        requestOrNull(api.getNetworkInterfaces(), 'interfaces'),
        requestOrNull(api.getRoamingStatus(), 'roaming'),
        requestOrNull(api.getCellsInfo(), 'cells'),
      ])

      // 慢速请求：不阻塞页面渲染，异步填充数据
      const statsPromise = requestOrNull(api.getSystemStats(), 'stats')
      const connectivityPromise = requestOrNull(api.getConnectivity(), 'connectivity')

      // 等待快速请求完成即可渲染页面
      const [
        deviceRes,
        simRes,
        networkRes,
        dataRes,
        airplaneModeRes,
        interfacesRes,
        roamingRes,
        cellsRes,
      ] = await fastPromise

      if (deviceRes?.data) setDeviceInfo(deviceRes.data)
      if (simRes?.data) setSimInfo(simRes.data)
      if (networkRes?.data) setNetworkInfo(networkRes.data)
      if (dataRes?.data) setDataStatus(dataRes.data.active)
      if (airplaneModeRes?.data) setAirplaneMode(airplaneModeRes.data)
      if (interfacesRes?.data) setConnectionAddresses(connectionAddressesFromInterfaces(interfacesRes.data.interfaces))
      if (roamingRes?.data) setRoaming(roamingRes.data)
      if (cellsRes?.data) setCellsInfo(cellsRes.data)

      // 快速数据就绪，立即解除 loading
      setInitialLoading(false)

      // 异步等待慢速请求并填充数据
      const [statsRes, connectivityRes] = await Promise.all([statsPromise, connectivityPromise])

      if (statsRes?.data) {
        setSystemStats(statsRes.data)
        updateSpeedHistory(statsRes.data)
      }
      if (connectivityRes?.data) setConnectivity(connectivityRes.data)

      const dataActive = dataRes?.data?.active ?? false
      if (statsRes?.data) {
        setQosInfo(qosFromWwanInterface(statsRes.data, dataActive))
      } else {
        setQosInfo(null)
      }
      if (failures.length > 0) {
        setError(failures[0])
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      setInitialLoading(false)
    }
  }, [updateSpeedHistory])

  const toggleData = useCallback(async () => {
    try {
      const nextStatus = !dataStatus
      await api.setDataStatus(nextStatus)
      setDataStatus(nextStatus)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [dataStatus])

  const toggleAirplaneMode = useCallback(async () => {
    const snapshot = airplaneMode
    const nextEnabled = !snapshot?.enabled
    if (snapshot) {
      setAirplaneMode({ ...snapshot, enabled: nextEnabled })
    }
    try {
      const response = await api.setAirplaneMode(nextEnabled)
      if (response.data) setAirplaneMode(response.data)
    } catch (err) {
      if (snapshot) setAirplaneMode(snapshot)
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [airplaneMode])

  const toggleRoaming = useCallback(async () => {
    try {
      const nextAllowed = !roaming?.roaming_allowed
      const response = await api.setRoamingAllowed(nextAllowed)
      if (response.data) setRoaming(response.data)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }, [roaming])

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      void loadData()
    }, 0)

    let interval: number | undefined
    if (refreshInterval > 0) {
      interval = window.setInterval(() => void loadData(), refreshInterval)
    }

    return () => {
      window.clearTimeout(timeout)
      if (interval !== undefined) {
        window.clearInterval(interval)
      }
    }
  }, [refreshInterval, refreshKey, loadData])

  return {
    initialLoading,
    error,
    setError,
    data: {
      deviceInfo,
      simInfo,
      systemStats,
      networkInfo,
      dataStatus,
      cellsInfo,
      qosInfo,
      airplaneMode,
      connectivity,
      connectionAddresses,
      speedHistory,
      roaming,
    } as DashboardData,
    actions: {
      toggleData,
      toggleAirplaneMode,
      toggleRoaming,
      loadData,
    } as DashboardActions,
  }
}
