import { Drawer, Grid, List, ListItem, ListItemButton, ListItemIcon, ListItemText } from "@mui/material"
import { Memory } from '@mui/icons-material'


export default ({ children }) => {
  const drawerWidth = 200

  return (
    <Grid container>
      <Drawer
        variant='permanent'
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: drawerWidth,
            boxSizing: 'border-box',
          }
        }}
      >
        <List>
          <ListItem disablePadding>
            <ListItemButton>
              <ListItemIcon>
                <Memory />
              </ListItemIcon>
              <ListItemText primary='Beliefs' />
            </ListItemButton>
          </ListItem>
        </List>
      </Drawer>
      <div style={{ width: `calc(100% - ${drawerWidth}px)`, padding: '16px' }}>
        { children }
      </div>
    </Grid>
  )
}
