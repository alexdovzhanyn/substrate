import { useState, useMemo } from 'react'
import { Card, Typography, Skeleton } from '@mui/material'
import classes from './styles.module.css'
import BeliefDetailDialog from './BeliefDetailDialog'

const POSSIBLE_BORDER_COLORS = [ '#ff5555', '#ffb86b', '#f1fa8c', '#50fa7b', '#8be9fd', '#bd93f9', '#ff79c8' ]
const MAX_SKELETON_LINES = 6

const uuidToInt = uuid => {
  let hash = 0

  for (let i = 0; i < uuid.length; i++) {
    hash = ((hash << 5) - hash + uuid.charCodeAt(i)) | 0
  }

  return hash >>> 0
}

export default ({ isLoading, belief, highlight }) => {
  const [ isDetailOpen, setIsDetailOpen ] = useState(false)

  const borderColor = useMemo(
    () => belief ? POSSIBLE_BORDER_COLORS[uuidToInt(belief.id) % POSSIBLE_BORDER_COLORS.length] : 'transparent',
    []
  )

  const highlightRegex = useMemo(() => new RegExp(`(${highlight})`, "gi"), [ highlight ])

  return (
    <>
      <Card
        className={ classes.BeliefCard }
        sx={{ borderTopColor: borderColor }}
        onClick={ () => setIsDetailOpen(true) }
      >
        { isLoading ? Array(Math.floor((Math.random() * MAX_SKELETON_LINES) + 1)).fill(0).map((_x, i) => (
          <Skeleton key={ i } animation="wave" width='100%' />
        )) : (
          <Typography variant='body2' sx={{ letterSpacing: 'normal' }}>
            { !highlight ? belief.content : (
              <span>
                { belief.content.split(highlightRegex).map(chunk => chunk.toLowerCase() == highlight.toLowerCase() 
                  ? <span className={ classes.HighlightedText }>{ chunk }</span>
                  : chunk
                )}
              </span>
            )}
          </Typography>
        )}
      </Card>
      { isDetailOpen && (
        <BeliefDetailDialog belief={ belief } onClose={ () => setIsDetailOpen(false) } />
      )}
    </>
  )
}
