import { mount } from 'svelte'
import './app.css'
import "../assets/gbboot.woff2"
import App from './App.svelte'

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
