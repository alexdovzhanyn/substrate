import { createRoot } from 'react-dom/client'
import App from './App.jsx'
import { store } from '@substrate/redux/store'
import { Provider } from 'react-redux'
import { BrowserRouter } from 'react-router'
import SnackbarProvider from '@substrate/containers/SnackbarProvider'
import { CssBaseline, ThemeProvider } from '@mui/material'
import draculaTheme from './assets/draculaTheme'

createRoot(document.getElementById('root')).render(
  <Provider store={ store }>
    <SnackbarProvider>
      <BrowserRouter>
        <ThemeProvider theme={ draculaTheme }>
          <CssBaseline />
          <App />
        </ThemeProvider>
      </BrowserRouter>
    </SnackbarProvider>
  </Provider>
)
