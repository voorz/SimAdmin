import { useState } from 'react'
import { Outlet } from 'react-router-dom'
import { Box, useMediaQuery, useTheme, type Theme } from '@mui/material'
import Sidebar from './Sidebar'
import TopBar from './TopBar'
import { RefreshContext } from '../../contexts/RefreshContext'
import { LAYOUT_BOTTOM_ACTION_BAR_HEIGHT, LAYOUT_BOTTOM_ACTION_BAR_ID } from './layoutConstants'

const DRAWER_WIDTH = 224
const DRAWER_MINI_WIDTH = 64
const LAYOUT_TRANSITION = '300ms cubic-bezier(0.4, 0, 0.2, 1)'

export default function MainLayout() {
  const theme = useTheme<Theme>()
  const isMobile = useMediaQuery(theme.breakpoints.down('sm'))
  const [mobileOpen, setMobileOpen] = useState(false)
  const [desktopOpen, setDesktopOpen] = useState(true) // 桌面端侧边栏状态，默认展开
  const [refreshInterval, setRefreshInterval] = useState(3000) // 默认 3 秒（移动端友好）
  const [refreshKey, setRefreshKey] = useState(0)

  const handleDrawerToggle = () => {
    if (isMobile) {
      setMobileOpen(!mobileOpen)
    } else {
      setDesktopOpen(!desktopOpen)
    }
  }

  const triggerRefresh = () => {
    setRefreshKey((prev) => prev + 1)
  }

  return (
    <RefreshContext.Provider
      value={{ refreshInterval, setRefreshInterval, refreshKey, triggerRefresh }}
    >
      <Box
        sx={{
          display: 'flex',
          height: '100vh',
          position: 'relative',
          overflow: 'hidden',
          bgcolor: 'background.default',
          '&::before': {
            content: '""',
            position: 'fixed',
            inset: 0,
            pointerEvents: 'none',
            zIndex: 0,
          },
        }}
      >
        {/* 侧边栏 */}
        <Sidebar
          drawerWidth={DRAWER_WIDTH}
          miniWidth={DRAWER_MINI_WIDTH}
          mobileOpen={mobileOpen}
          desktopOpen={desktopOpen}
          onClose={handleDrawerToggle}
          isMobile={isMobile}
        />

        <Box
          sx={{
            display: 'flex',
            flexDirection: 'column',
            flexGrow: 1,
            minWidth: 0,
            height: '100vh',
            position: 'relative',
            zIndex: 1,
            transition: `width ${LAYOUT_TRANSITION}`,
            willChange: 'width',
          }}
        >
          {/* 顶部导航栏 */}
          <TopBar
            drawerWidth={desktopOpen ? DRAWER_WIDTH : DRAWER_MINI_WIDTH}
            onMenuClick={handleDrawerToggle}
          />

          <Box
            sx={{
              borderBottom: '1px solid',
              borderColor: (currentTheme) => currentTheme.palette.mode === 'light'
                ? 'rgba(0,0,0,0.08)'
                : 'rgba(255,255,255,0.1)',
              flexShrink: 0,
            }}
          />

          {/* 主内容区 */}
          <Box
            component="main"
            sx={{
              flexGrow: 1,
              minHeight: 0,
              overflow: 'auto',
              p: { xs: 2, sm: 3 },
            }}
          >
            <Outlet />
          </Box>

          <Box
            id={LAYOUT_BOTTOM_ACTION_BAR_ID}
            sx={(currentTheme) => ({
              flexShrink: 0,
              '&:empty': {
                display: 'none',
              },
              '&:not(:empty)': {
                alignItems: 'center',
                bgcolor: 'transparent',
                borderTop: '1px solid',
                borderColor: currentTheme.palette.mode === 'light'
                  ? 'rgba(0,0,0,0.08)'
                  : 'rgba(255,255,255,0.1)',
                display: 'flex',
                height: LAYOUT_BOTTOM_ACTION_BAR_HEIGHT,
                minHeight: LAYOUT_BOTTOM_ACTION_BAR_HEIGHT,
                px: { xs: 2, sm: 3 },
              },
            })}
          />
        </Box>
      </Box>
    </RefreshContext.Provider>
  )
}
