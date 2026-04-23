import { useState, useMemo } from 'react'
import { Box, Drawer, Grid, List, ListItem, ListItemButton, ListItemIcon, ListItemText, IconButton, useTheme, Divider } from "@mui/material"
import { Memory, ChevronLeft as ChevronLeftIcon, ChevronRight as ChevronRightIcon } from '@mui/icons-material'


export default ({ children }) => {
  const [ isDrawerOpen, setIsDrawerOpen ] = useState(false)
  const theme = useTheme()
  const drawerWidthOpen = 200
  const drawerWidthClosed = 50

  const drawerWidth = useMemo(() => isDrawerOpen ? drawerWidthOpen : drawerWidthClosed, [ isDrawerOpen ])

  return (
    <Grid container>
      <Drawer
        variant='permanent'
        open={ isDrawerOpen }
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          transition: theme.transitions.create('width', {
            easing: theme.transitions.easing.sharp,
            duration: isDrawerOpen ? theme.transitions.duration.leavingScreen : theme.transitions.duration.enteringScreen,
          }),
          '& .MuiDrawer-paper': {
            width: drawerWidth,
            boxSizing: 'border-box',
            transition: theme.transitions.create('width', {
              easing: theme.transitions.easing.sharp,
              duration: isDrawerOpen ? theme.transitions.duration.leavingScreen : theme.transitions.duration.enteringScreen,
            })
          }
        }}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: isDrawerOpen ? 'flex-end' : 'center', padding: '4px 0px' }}>
          <IconButton onClick={ () => setIsDrawerOpen(!isDrawerOpen) }>
            { isDrawerOpen ? <ChevronLeftIcon /> : <ChevronRightIcon /> }
          </IconButton>
        </Box>
        <Divider />
        <List>
          <ListItem disablePadding>
            <ListItemButton sx={{ justifyContent: isDrawerOpen ? 'initial' : 'center' }}>
              <ListItemIcon sx={{ justifyContent: isDrawerOpen ? 'initial' : 'center' }}>
                <Memory />
              </ListItemIcon>
              { isDrawerOpen && <ListItemText primary='Beliefs' /> }
            </ListItemButton>
          </ListItem>
        </List>
      </Drawer>
      <Box sx={{
        width: `calc(100% - ${drawerWidth}px)`,
        height: '100vh',
        transition: theme.transitions.create(['width', 'margin'], {
          easing: theme.transitions.easing.sharp,
          duration: isDrawerOpen ? theme.transitions.duration.leavingScreen : theme.transitions.duration.enteringScreen,
        }),
      }}>
        { children }
      </Box>
    </Grid>
  )
}
