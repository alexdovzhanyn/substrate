import AppLayout from '@substrate/containers/AppLayout'
import { Routes, Route } from 'react-router'
import BeliefPage from '@substrate/pages/BeliefPage'
import './App.css'

function App() {
  return (
    <AppLayout>
      <Routes>
        <Route index element={ <BeliefPage /> } />
      </Routes>
    </AppLayout>
  )
}

export default App
