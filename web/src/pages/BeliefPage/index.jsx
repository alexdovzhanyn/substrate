import { useState, useEffect, useCallback, useRef } from 'react'
import { useDispatch } from 'react-redux'
import { fetchBeliefs } from '@substrate/redux/beliefSlice'
import { Grid, TextField, InputAdornment, SpeedDial, SpeedDialAction } from '@mui/material'
import {
  Search as SearchIcon,
  BugReport as BugReportIcon,
  Add as AddIcon,
  Construction as ConstructionIcon,
  Delete as DeleteIcon
} from '@mui/icons-material'
import { debounce } from '../../util/util'
import BeliefList from './BeliefList'

export default () => {
  const dispatch = useDispatch()
  const [ filterState, setFilterState ] = useState({ search: '', page: 1 })
  const isLazyLoadingRef = useRef(false)

  useEffect(() => {
    dispatch(fetchBeliefs({ search: '', page: 1 }))
  }, [])

  const triggerSearch = useCallback(debounce(newFilterState => {
    dispatch(fetchBeliefs(newFilterState))
    setFilterState(newFilterState)
    isLazyLoadingRef.current = false
  }, 500), [])

  const onSearchUpdate = ({ target: { value } }) => triggerSearch({ search: value, page: 1 })

  const toggleQueryDebugger = () => {}

  const onLazyLoad = () => {
    if (isLazyLoadingRef.current) return

    isLazyLoadingRef.current = true
    triggerSearch({ ...filterState, page: filterState.page + 1})
  }

  return (
    <Grid container sx={{ height: '100%', flexDirection: 'column' }}>
      <Grid size={ 12 } sx={{ padding: '16px' }}>
        <Grid container>
          <Grid size={ 6 }>
            <TextField
              fullWidth
              placeholder='Search by keyword...'
              size='small'
              onChange={ onSearchUpdate }
              slotProps={{
                input: {
                  startAdornment: (
                    <InputAdornment position='start'>
                      <SearchIcon />
                    </InputAdornment>
                  )
                }
              }}
            />
          </Grid>
          <Grid size={ 6 } sx={{ display: 'flex', justifyContent: 'flex-end', position: 'relative', alignItems: 'center' }}>
            <SpeedDial
              ariaLabel='Actions'
              direction='left'
              icon={ <ConstructionIcon /> }
              FabProps={{ color: 'success', size: 'medium' }}
              sx={{ position: 'absolute' }}
            >
              <SpeedDialAction
                icon={ <AddIcon /> }
                slotProps={{ tooltip: { title: 'New belief' } }}
                onClick={ () => {} }
              />
              <SpeedDialAction
                icon={ <BugReportIcon /> }
                slotProps={{ tooltip: { title: 'Query debugger' } }}
                onClick={ toggleQueryDebugger }
              />
              <SpeedDialAction
                icon={ <DeleteIcon /> }
                slotProps={{ tooltip: { title: 'Flush beliefs' } }}
                onClick={ () => {} }
              />
            </SpeedDial>
          </Grid>
        </Grid>
      </Grid>
      <BeliefList onLazyLoad={ onLazyLoad } highlight={ filterState.search } />
    </Grid>
  )
}
