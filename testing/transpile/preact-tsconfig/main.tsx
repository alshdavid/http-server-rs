import { h, render } from 'preact'
import { App } from './app.tsx'

render(<App title={"Hello World"} />, document.getElementById('root')!);
