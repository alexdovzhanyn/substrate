import { SnackbarProvider, enqueueSnackbar } from 'notistack'
import { useEffect } from 'react'
import { useSelector } from 'react-redux' 

export default ({ children }) => {
  const notification = useSelector(state => state.notifications)

  useEffect(() => {
    if (!notification.msg) return

    enqueueSnackbar(notification.msg, { variant: notification.type, ...(notification.option || {}) })
  }, [ notification ])

  return (
    <>
      <SnackbarProvider anchorOrigin={{ horizontal: 'right', vertical: 'top' }}/>
      { children }
    </>
  )
}
