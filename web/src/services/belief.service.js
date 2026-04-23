import { API_URL } from './api'

export const BELIEF_PAGE_SIZE = 50

export const apiFetchBeliefs = ({ search, page }) => {
  return fetch(`${API_URL}/beliefs?search=${search}&offset=${BELIEF_PAGE_SIZE * (page - 1)}`)
}
