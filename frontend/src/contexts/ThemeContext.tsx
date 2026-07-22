/* eslint-disable react-refresh/only-export-components */
import { createContext, useContext, useState, useEffect } from 'react'
import type { ReactNode } from 'react'
import { ThemeProvider as MuiThemeProvider, createTheme } from '@mui/material/styles'
import CssBaseline from '@mui/material/CssBaseline'

type ThemeMode = 'light' | 'dark'

interface ThemeContextType {
  mode: ThemeMode
  toggleTheme: () => void
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined)

export function useTheme() {
  const context = useContext(ThemeContext)
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider')
  }
  return context
}

interface ThemeProviderProps {
  children: ReactNode
}

export function ThemeProvider({ children }: ThemeProviderProps) {
  // 从 localStorage 读取保存的主题，默认浅色
  const [mode, setMode] = useState<ThemeMode>(() => {
    const saved = localStorage.getItem('theme-mode')
    return (saved === 'dark' ? 'dark' : 'light') as ThemeMode
  })

  // 保存主题设置到 localStorage
  useEffect(() => {
    localStorage.setItem('theme-mode', mode)
  }, [mode])

  const toggleTheme = () => {
    setMode((prev) => (prev === 'light' ? 'dark' : 'light'))
  }

  const theme = createTheme({
    palette: {
      mode,
      primary: {
        main: mode === 'light' ? '#1a1a1a' : '#e5e5e5',
        light: mode === 'light' ? '#4a4a4a' : '#f5f5f5',
        dark: mode === 'light' ? '#000000' : '#b5b5b5',
      },
      secondary: {
        main: mode === 'light' ? '#6a6a6a' : '#9a9a9a',
        light: mode === 'light' ? '#9a9a9a' : '#c0c0c0',
        dark: mode === 'light' ? '#4a4a4a' : '#6a6a6a',
      },
      success: {
        main: mode === 'light' ? '#2aae67' : '#66bb6a',
        light: mode === 'light' ? '#4caf50' : '#81c784',
        dark: mode === 'light' ? '#1b5e20' : '#388e3c',
        contrastText: '#ffffff',
      },
      warning: {
        main: mode === 'light' ? '#ed6c02' : '#ffa726',
        light: mode === 'light' ? '#ff9800' : '#ffb74d',
        dark: mode === 'light' ? '#e65100' : '#f57c00',
      },
      error: {
        main: mode === 'light' ? '#ef4444' : '#f44336',
        light: mode === 'light' ? '#ef5350' : '#e57373',
        dark: mode === 'light' ? '#c62828' : '#ef4444',
      },
      info: {
        main: mode === 'light' ? '#0288d1' : '#29b6f6',
        light: mode === 'light' ? '#03a9f4' : '#4fc3f7',
        dark: mode === 'light' ? '#01579b' : '#0277bd',
      },
      background: {
        default: mode === 'light' ? '#f5f5f5' : '#1a1a1a',
        paper: mode === 'light' ? 'rgba(255,255,255,0.72)' : 'rgba(30,30,30,0.72)',
      },
      text: {
        primary: mode === 'light' ? '#0f172a' : '#e5edf7',
        secondary: mode === 'light' ? '#475569' : '#94a3b8',
        disabled: mode === 'light' ? '#94a3b8' : '#64748b',
      },
      divider: mode === 'light' ? 'rgba(0,0,0,0.12)' : 'rgba(255,255,255,0.12)',
    },
    typography: {
      fontFamily: [
        '-apple-system',
        'BlinkMacSystemFont',
        '"Segoe UI"',
        'Roboto',
        '"Helvetica Neue"',
        'Arial',
        'sans-serif',
        '"Apple Color Emoji"',
        '"Segoe UI Emoji"',
        '"Segoe UI Symbol"',
      ].join(','),
      h4: {
        fontWeight: 600,
      },
      h5: {
        fontWeight: 600,
      },
      h6: {
        fontWeight: 600,
      },
    },
    components: {
      MuiCssBaseline: {
        styleOverrides: {
          html: {
            minWidth: 320,
            minHeight: '100%',
          },
          body: {
            margin: 0,
            minWidth: 320,
            minHeight: '100vh',
            backgroundColor: mode === 'light' ? '#f5f5f5' : '#1a1a1a',
            scrollbarColor: mode === 'dark' ? '#4a4a4a transparent' : '#c0c0c0 transparent',
            '&::-webkit-scrollbar, & *::-webkit-scrollbar': {
              width: 6,
              height: 6,
            },
            '&::-webkit-scrollbar-thumb, & *::-webkit-scrollbar-thumb': {
              borderRadius: 999,
              backgroundColor: mode === 'dark' ? '#4a4a4a' : '#c0c0c0',
            },
            '&::-webkit-scrollbar-track, & *::-webkit-scrollbar-track': {
              backgroundColor: 'transparent',
            },
            '& #root': {
              minHeight: '100vh',
            },
          },
        },
      },
      MuiCard: {
        defaultProps: {
          elevation: 0,
        },
        styleOverrides: {
          root: {
            borderRadius: 12,
            backgroundImage: 'none',
            backgroundColor: mode === 'light' ? 'rgba(255,255,255,0.68)' : 'rgba(30,30,30,0.72)',
            border: mode === 'light' ? '1px solid rgba(255,255,255,0.82)' : '1px solid rgba(255,255,255,0.12)',
            boxShadow: mode === 'light'
              ? '0 18px 42px -28px rgba(0,0,0,0.18), 0 1px 0 rgba(255,255,255,0.8) inset'
              : '0 18px 42px -30px rgba(0,0,0,0.72), 0 1px 0 rgba(255,255,255,0.06) inset',
            backdropFilter: 'blur(22px)',
            WebkitBackdropFilter: 'blur(22px)',
          },
        },
      },
      MuiButton: {
        defaultProps: {
          disableElevation: true,
        },
        styleOverrides: {
          root: {
            borderRadius: 8,
            textTransform: 'none',
            fontWeight: 500,
          },
        },
      },
      MuiPaper: {
        styleOverrides: {
          root: {
            borderRadius: 12,
            backgroundImage: 'none',
            backgroundColor: mode === 'light' ? 'rgba(255,255,255,0.66)' : 'rgba(30,30,30,0.68)',
            borderColor: mode === 'light' ? 'rgba(226,232,240,0.75)' : 'rgba(255,255,255,0.12)',
            backdropFilter: 'blur(20px)',
            WebkitBackdropFilter: 'blur(20px)',
          },
        },
      },
      MuiAppBar: {
        styleOverrides: {
          root: {
            borderRadius: 0,
            backgroundImage: 'none',
            boxShadow: 'none',
          },
        },
      },
      MuiChip: {
        styleOverrides: {
          root: {
            borderRadius: 8,
            fontWeight: 500,
          },
        },
      },
      MuiAlert: {
        styleOverrides: {
          filledSuccess: {
            color: '#ffffff !important',
            '& .MuiAlert-icon': {
              color: '#ffffff !important',
            },
            '& .MuiAlert-message': {
              color: '#ffffff !important',
            },
            '& .MuiAlert-action .MuiIconButton-root': {
              color: '#ffffff !important',
            },
          },
        },
      },
    },
    shape: {
      borderRadius: 8,
    },
  })

  return (
    <ThemeContext.Provider value={{ mode, toggleTheme }}>
      <MuiThemeProvider theme={theme}>
        <CssBaseline />
        {children}
      </MuiThemeProvider>
    </ThemeContext.Provider>
  )
}
