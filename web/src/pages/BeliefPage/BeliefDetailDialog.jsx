import { Box, Grid, Dialog, DialogTitle, DialogContent, Typography, Chip, List, ListItem, ListItemIcon, ListItemText, Card } from "@mui/material"
import { Sell as SellIcon, Fingerprint as FingerprintIcon } from '@mui/icons-material'
import classes from './styles.module.css'
import moment from 'moment'

export default ({ belief, onClose }) => {
  return (
    <Dialog
      open
      onClose={ onClose }
    >
      <DialogTitle sx={{ fontWeight: 'bold' }}>Belief <span style={{ color: '#50FA7B' }}>{ belief.id }</span></DialogTitle>
      <Grid container sx={{ padding: '0px 24px 4px', marginTop: '-14px' }}>
        <Grid size={ 6 }>
          <Typography variant='caption'>
            Created: { moment.unix(belief.created_at).format('MMM Do, YYYY h:mma') } by { belief.created_by }
          </Typography>
        </Grid>
        { belief.created_at != belief.updated_at && (
          <Grid size={ 6 } sx={{ display: 'flex', justifyContent: 'flex-end', alignItems: 'center' }}>
            <Typography variant='caption'>
              Last updated: {  moment.unix(belief.updated_at).format('MMM Do, YYYY h:mma') }
            </Typography>
          </Grid>
        )}
      </Grid>
      <DialogContent dividers>
        <Grid container rowSpacing={ 2 }>
          <Grid size={ 12 }>
            <Box className={ classes.BeliefDetailContentContainer }>
              <Typography variant='body1'>{ belief.content }</Typography>
            </Box>
            
          </Grid>
          <Grid size={ 12 }>
            <Typography variant='body1' sx={{ fontWeight: 'bold' }}>Embedded Queries</Typography>
            <List dense disablePadding>
              { belief.possible_queries.map((query, i) => (
                <ListItem key={ query } className={ classes.EmbeddingQueryContainer }>
                  <ListItemIcon>
                    <FingerprintIcon sx={{ fontSize: '20px' }} />
                  </ListItemIcon>
                  <ListItemText primary={ query } />
                </ListItem>
              ))}
            </List>
          </Grid>
        </Grid>
      </DialogContent>
      <Box sx={{ display: 'flex', alignContent: 'center', columnGap: 1, padding: '8px 24px'}}>
        { belief.tags.map(tag => (
          <Chip key={ tag } icon={ <SellIcon /> } label={ tag } color='secondary' size='small' sx={{ borderRadius: '4px' }} />
        )) }
      </Box>
    </Dialog>
  )
}
