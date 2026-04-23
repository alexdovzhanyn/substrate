import { useSelector } from 'react-redux'
import { Box, Grid, Typography, useMediaQuery, useTheme } from '@mui/material'
import classes from './styles.module.css'
import { Memory as MemoryIcon } from '@mui/icons-material'
import { Masonry } from '@mui/lab'
import BeliefCard from './BeliefCard'

export default ({ onLazyLoad, highlight }) => {
  const { records, hasMore, isLoading } = useSelector(state => state.beliefs)

  const theme = useTheme()
  const isLg = useMediaQuery(theme.breakpoints.up('lg'))
  const isXl = useMediaQuery(theme.breakpoints.up('xl'))

  const onScroll = ({ target }) => {
    const lazyLoadTriggerDistance = 200
    const distanceFromBottom = target.scrollHeight - target.scrollTop - target.clientHeight

    if (distanceFromBottom > lazyLoadTriggerDistance || !hasMore) return

    onLazyLoad()
  }

  if (!isLoading && !records.length) {
    return (
      <Grid container sx={{ display: 'flex', justifyContent: 'center' }}>
        <Grid size={ isXl ? 6 : ( isLg ? 9 : 12 ) } className={ classes.BeliefListEmptyContainer }>
          <Box className={ classes.BeliefListEmptyStateWrapper } sx={{ borderColor: 'text.secondary' }}>
            <MemoryIcon sx={{ fontSize: 100, color: 'text.secondary' }} />
            <Typography variant='body1' color='textSecondary'>
              { highlight 
                ? 'No beliefs contain the searched keywords'
                : 'Beliefs will show up here once some have been recorded'
              }
            </Typography>
          </Box>
        </Grid>
      </Grid>
    )
  }

  return (
    <Grid size={ 12 } sx={{ padding: '16px', overflowY: 'auto', flex: 1 }} onScroll={ onScroll }>
      <Masonry columns={ isXl ? 6 : (isLg ? 5 : 4) } spacing={ 2 } sx={{ width: 'auto' }}>
        { records.map(r => (
          <BeliefCard key={ r.id } belief={ r } highlight={ highlight } /> 
        ))}
        { isLoading && Array(50).fill(0).map((_x, i) => (
          <BeliefCard key={ i } isLoading />
        )) }
      </Masonry>
    </Grid>
  )
}
