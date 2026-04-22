import AppLayout from '@substrate/containers/AppLayout'
import { Routes, Route } from 'react-router'
import BeliefList from '@substrate/pages/BeliefList'
import './App.css'

function App() {
  return (
    <AppLayout>
      <Routes>
        <Route index element={ <BeliefList /> } />
      </Routes>
    </AppLayout>
  )
}

export default App
