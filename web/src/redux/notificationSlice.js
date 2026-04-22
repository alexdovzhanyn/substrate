import { createSlice } from '@reduxjs/toolkit'

const initialState = {
  msg: null,
  type: null,
  option: null
}

export const notificationSlice = createSlice({
  name: 'notifications',
  initialState,
  reducers: {
    showNotification: (state, { payload }) => {
      state.msg = payload.msg
      state.type = payload.type
      state.option = payload?.option ? payload.option : null
      return state
    },
    clearState: (state) => {
      state.msg = null
      state.type = null
      state.option = null
      return state
    },
  },
})

export const { showNotification, clearState } = notificationSlice.actions

export default notificationSlice.reducer
