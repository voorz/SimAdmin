import { type ElementType } from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import {
  Box,
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Toolbar,
  Tooltip,
  Typography,
} from '@mui/material'
import {
  Dashboard as DashboardIcon,
  SignalCellularAlt as SignalIcon,
  Settings as SettingsIcon,
  Sms as SmsIcon,
  NotificationsActive as NotificationsIcon,
  GitHub as GitHubIcon,
  SystemUpdateAlt as OtaIcon,
  Router as RouterIcon,
  SimCard as SimIcon,
  AutoMode as AutomationIcon,
  Shield as SecurityIcon,
} from '@mui/icons-material'

const SIDEBAR_TRANSITION = '300ms cubic-bezier(0.4, 0, 0.2, 1)'

interface SidebarProps {
  drawerWidth: number
  miniWidth: number
  mobileOpen: boolean
  desktopOpen: boolean
  onClose: () => void
  isMobile: boolean
}

interface MenuItem {
  path: string
  label: string
  icon: ElementType
}

const menuItems: MenuItem[] = [
  { path: '/', label: '仪表盘', icon: DashboardIcon },
  { path: '/sim', label: 'SIM 卡', icon: SimIcon },
  { path: '/sms', label: '短信管理', icon: SmsIcon },
  { path: '/network', label: '蜂窝网络', icon: SignalIcon },
  { path: '/device-network', label: '设备网络', icon: RouterIcon },
  { path: '/automation', label: '自动化中心', icon: AutomationIcon },
  { path: '/notifications', label: '通知中心', icon: NotificationsIcon },
  { path: '/config', label: '基本配置', icon: SettingsIcon },
  { path: '/config/security', label: '安全性', icon: SecurityIcon },
  { path: '/ota', label: 'OTA 更新', icon: OtaIcon },
]

export default function Sidebar({
  drawerWidth,
  miniWidth,
  mobileOpen,
  desktopOpen,
  onClose,
  isMobile,
}: SidebarProps) {
  const navigate = useNavigate()
  const location = useLocation()

  const handleNavigation = (path: string): void => {
    void navigate(path)
    if (isMobile) onClose()
  }

  const renderDrawer = (compact = false) => (
    <Box sx={{ display: 'flex', flexDirection: 'column', height: '100%', overflow: 'hidden' }}>
      <Toolbar
        sx={{
          minHeight: 64,
          px: 0,
        }}
      >
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: compact ? 'center' : 'flex-start',
            gap: 1.25,
            width: '100%',
            minWidth: 0,
          }}
        >
          <Box
            component="img"
            src="/simadmin-logo.svg"
            alt="SimAdmin"
            sx={{
              width: 38,
              height: 38,
              mx: compact ? 0 : '13px',
              flexShrink: 0,
              display: 'block',
              transition: `margin ${SIDEBAR_TRANSITION}, width ${SIDEBAR_TRANSITION}, height ${SIDEBAR_TRANSITION}`,
            }}
          />
          <Typography
            variant="h6"
            noWrap
            component="div"
            fontWeight={700}
            sx={{
              opacity: compact ? 0 : 1,
              maxWidth: compact ? 0 : 132,
              transform: compact ? 'translateX(-6px)' : 'translateX(0)',
              overflow: 'hidden',
              transition: `opacity 180ms ease, max-width ${SIDEBAR_TRANSITION}, transform ${SIDEBAR_TRANSITION}`,
            }}
          >
            SimAdmin
          </Typography>
        </Box>
      </Toolbar>
      <List sx={{ flexGrow: 1, py: 1.5, px: compact ? 0.75 : 1, overflowY: 'auto', overflowX: 'hidden' }}>
        {menuItems.map((item) => {
          const selected = location.pathname === item.path
          const IconComponent = item.icon
          return (
            <ListItem key={item.path} disablePadding>
              {compact ? (
                <Tooltip title={item.label} placement="right">
                  <ListItemButton
                    selected={selected}
                    onClick={() => handleNavigation(item.path)}
                    sx={{
                      minHeight: 44,
                      borderRadius: 1.5,
                      justifyContent: 'center',
                      px: 0,
                      mb: 0.5,
                      color: selected ? 'primary.main' : 'text.secondary',
                      '&.Mui-selected': {
                        bgcolor: (theme) => theme.palette.mode === 'light' ? 'rgba(255,255,255,0.68)' : 'rgba(30,30,30,0.72)',
                        boxShadow: '0 8px 22px -18px rgba(18,150,219,0.6)',
                        borderRight: '2px solid',
                        borderColor: 'primary.main',
                      },
                      '&.Mui-selected:hover, &:hover': {
                        bgcolor: (theme) => theme.palette.mode === 'light' ? 'rgba(255,255,255,0.58)' : 'rgba(30,30,30,0.64)',
                      },
                    }}
                  >
                    <ListItemIcon
                      sx={{
                        minWidth: 0,
                        color: 'inherit',
                        justifyContent: 'center',
                      }}
                    >
                      <IconComponent sx={{ fontSize: 20 }} />
                    </ListItemIcon>
                  </ListItemButton>
                </Tooltip>
              ) : (
                <ListItemButton
                  selected={selected}
                  onClick={() => handleNavigation(item.path)}
                  sx={{
                    minHeight: 44,
                    borderRadius: 1.5,
                    justifyContent: 'flex-start',
                    px: 0,
                    mb: 0.5,
                    color: selected ? 'primary.main' : 'text.secondary',
                    '&.Mui-selected': {
                      bgcolor: (theme) => theme.palette.mode === 'light' ? 'rgba(255,255,255,0.68)' : 'rgba(30,30,30,0.72)',
                      boxShadow: '0 8px 22px -18px rgba(18,150,219,0.6)',
                      borderRight: '2px solid',
                      borderColor: 'primary.main',
                    },
                    '&.Mui-selected:hover, &:hover': {
                      bgcolor: (theme) => theme.palette.mode === 'light' ? 'rgba(255,255,255,0.58)' : 'rgba(30,30,30,0.64)',
                    },
                  }}
                >
                  <ListItemIcon
                    sx={{
                      minWidth: 38,
                      width: 48,
                      color: 'inherit',
                      justifyContent: 'center',
                      flexShrink: 0,
                    }}
                  >
                    <IconComponent sx={{ fontSize: 20 }} />
                  </ListItemIcon>
                  <ListItemText
                    primary={item.label}
                    primaryTypographyProps={{ noWrap: true, fontSize: 14, fontWeight: selected ? 700 : 500 }}
                  />
                </ListItemButton>
              )}
            </ListItem>
          )
        })}
      </List>

      <Box
        sx={{
          p: compact ? 1 : 2,
          borderTop: 1,
          borderColor: 'divider',
          display: 'flex',
          flexDirection: 'column',
          alignItems: compact ? 'center' : 'flex-start',
        }}
      >
        <Box
          component="a"
          href="https://github.com/voorz/SimAdmin"
          target="_blank"
          rel="noopener noreferrer"
          sx={{
            display: 'flex',
            alignItems: 'center',
            gap: compact ? 0 : 0.5,
            color: 'text.secondary',
            fontSize: '0.75rem',
            textDecoration: 'none',
            width: 'fit-content',
            '&:hover': { color: 'primary.main' },
          }}
        >
          <GitHubIcon sx={{ fontSize: compact ? 22 : 16 }} />
          <Typography
            variant="caption"
            color="inherit"
            sx={{
              opacity: compact ? 0 : 1,
              maxWidth: compact ? 0 : 110,
              overflow: 'hidden',
              whiteSpace: 'nowrap',
              transition: `opacity ${SIDEBAR_TRANSITION}, max-width ${SIDEBAR_TRANSITION}`,
            }}
          >
            voorz/SimAdmin
          </Typography>
        </Box>
        <Box
          sx={{
            opacity: compact ? 0 : 1,
            maxHeight: compact ? 0 : 48,
            overflow: 'hidden',
            transition: `opacity ${SIDEBAR_TRANSITION}, max-height ${SIDEBAR_TRANSITION}`,
          }}
        >
          <Typography variant="caption" color="text.disabled" sx={{ display: 'block', mt: 0.5 }}>
            v{__APP_VERSION__} ({__GIT_BRANCH__}/{__GIT_COMMIT__})
          </Typography>
          <Typography variant="caption" color="text.disabled" sx={{ display: 'block', mt: 0.5 }}>
            Copyright © 2026 @voorz
          </Typography>
        </Box>
      </Box>
    </Box>
  )

  const paperSx = {
    boxSizing: 'border-box',
    borderRadius: 0,
    borderRight: '1px solid',
    borderColor: 'divider',
    bgcolor: (theme: import('@mui/material').Theme) => theme.palette.mode === 'light' ? 'rgba(255,255,255,0.42)' : 'rgba(30,30,30,0.54)',
    boxShadow: '4px 0 24px -16px rgba(0,0,0,0.18)',
    backdropFilter: 'blur(28px)',
    WebkitBackdropFilter: 'blur(28px)',
  } as const

  return (
    <Box
      component="nav"
      sx={{
        width: { xs: 0, sm: desktopOpen ? drawerWidth : miniWidth },
        flexShrink: { sm: 0 },
        transition: `width ${SIDEBAR_TRANSITION}`,
        willChange: 'width',
      }}
    >
      <Drawer
        variant="temporary"
        open={mobileOpen}
        onClose={onClose}
        ModalProps={{ keepMounted: true }}
        sx={{
          display: { xs: 'block', sm: 'none' },
          '& .MuiDrawer-paper': { ...paperSx, width: drawerWidth },
        }}
      >
        {renderDrawer(false)}
      </Drawer>

      <Drawer
        variant="persistent"
        open
        sx={{
          display: { xs: 'none', sm: 'block' },
          '& .MuiDrawer-paper': {
            ...paperSx,
            width: desktopOpen ? drawerWidth : miniWidth,
            overflowX: 'hidden',
            transition: `width ${SIDEBAR_TRANSITION}`,
            willChange: 'width',
          },
        }}
      >
        {renderDrawer(!desktopOpen)}
      </Drawer>
    </Box>
  )
}
