import { createAsyncThunk, createSlice } from '@reduxjs/toolkit'
import { apiFetchBeliefs, BELIEF_PAGE_SIZE } from '@substrate/services/belief.service'
import { showNotification } from '@substrate/redux/notificationSlice'

export const fetchBeliefs = createAsyncThunk(
  'beliefs/fetchBeliefs',
  async (beliefQuery, { dispatch, rejectWithValue }) => {
    try {
      const response = await apiFetchBeliefs(beliefQuery)

      if (!response.ok) {
        dispatch(showNotification({ msg: "Could not load beliefs", type: 'error' }))
        return rejectWithValue("Could not load beliefs")
      }

      return await response.json();
    } catch (err) {
      console.error(err.message)
      dispatch(showNotification({ msg: "Could not load beliefs", type: 'error' }))

      return rejectWithValue('Could not load beliefs')
    }
  }
)

const initialState = {
  records: [],
  hasMore: true,
  isLoading: false
}

export const beliefSlice = createSlice({
  name: 'beliefs',
  initialState,
  reducers: {},
  extraReducers: builder => {
    builder
      .addCase(fetchBeliefs.pending, state => {
        state.isLoading = true
      })
      .addCase(fetchBeliefs.fulfilled, (state, { payload, meta }) => {
        state.isLoading = false

        console.log(meta)
        if (meta.arg.page == 1) {
          state.records = payload.beliefs
        } else {
          state.records = [ ...state.records, ...payload.beliefs ]
        }

        state.hasMore = payload.beliefs.length >= BELIEF_PAGE_SIZE 
      })
      .addCase(fetchBeliefs.rejected, state => {
        state.isLoading = false
      })
  }
})

export default beliefSlice.reducer
