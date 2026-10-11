import { h, render, Fragment, type ComponentType } from 'preact'
import { useState, useEffect } from 'preact/hooks'
import { AboutPage } from './about.tsx';
import { HomePage } from './home.tsx';

function App() {
  const [route, setRoute] = useState<ComponentType | null>(null)

  function runRouter() {
    const route = window.location.hash.replace('#', '')
    switch (route) {
      case 'home':
        setRoute(() => HomePage)
        break
      case 'about':
        setRoute(() => AboutPage)
        break
      default:
        setRoute(() => null)
    }
  }

  useEffect(() => {
    if (!window.location.hash) window.location.hash = 'home'
    runRouter()
    globalThis.addEventListener('hashchange', runRouter)
    return () => globalThis.removeEventListener('hashchange', runRouter)
  }, [])

  return <Fragment>{route && h(route, {})}</Fragment>
}

render(<App />, document.getElementById("root")!);
