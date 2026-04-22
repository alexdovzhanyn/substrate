import { useState, useEffect } from 'react'
import { useSelector, useDispatch } from 'react-redux'
import { fetchBeliefs } from '@substrate/redux/beliefSlice'
import { Box, Grid, Card, List, ListItemText, ListItemButton, ListItem, Typography, Skeleton } from '@mui/material'
import classes from './styles.module.css'
import { Memory } from '@mui/icons-material'

export default () => {
  const dispatch = useDispatch()
  const { records, isLoading } = useSelector(state => state.beliefs)

  useEffect(() => {
    dispatch(fetchBeliefs({ search: '', page: 1 }))
  }, [])

  return (
    <Grid container>
      <Grid size={ 6 }>
        <Card>
          { isLoading ? (
            <List>
              { Array(10).fill(0).map((_x, i) => (
                <ListItem key={ i }>
                  <Skeleton animation="wave" height={ 40 } width='100%' />
                </ListItem>
              )) }
            </List>
          ) : records.length ? (
            <List>
              { records.map(r => (
                <ListItem key={ r.id }>
                  <ListItemButton>
                    <ListItemText primary={ r.content } slotProps={{ primary: { noWrap: true } }} />
                  </ListItemButton>
                </ListItem>
              )) }
            </List>
          ) : (
            <Grid container className={ classes.BeliefListEmptyContainer }>
              <Box className={ classes.BeliefListEmptyStateWrapper } sx={{ borderColor: 'text.secondary' }}>
                <Memory sx={{ fontSize: 100, color: 'text.secondary' }} />
                <Typography variant='body1' color='textSecondary'>Beliefs will show up here once some have been recorded</Typography>
              </Box>
            </Grid>
          )}
        </Card>
      </Grid>

    </Grid>
  )
}
