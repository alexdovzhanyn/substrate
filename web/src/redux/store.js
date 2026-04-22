import { configureStore } from '@reduxjs/toolkit'
import beliefReducer from '@substrate/redux/beliefSlice'
import notificationReducer from '@substrate/redux/notificationSlice'

export const store = configureStore({
  reducer: {
    beliefs: beliefReducer,
    notifications: notificationReducer
  }
})
