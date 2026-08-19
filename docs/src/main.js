import { createSSRApp } from 'vue'
import App from './App.vue'
import './style.css'

export function createApp() {
  return createSSRApp(App)
}
