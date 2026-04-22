import { API_URL } from './api'

export const apiFetchBeliefs = ({ search, page }) => fetch(`${API_URL}/beliefs`)
