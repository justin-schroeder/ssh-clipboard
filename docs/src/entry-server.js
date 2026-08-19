import { renderToString } from '@vue/server-renderer'
import { createApp } from './main.js'

export function render() {
  return renderToString(createApp())
}
