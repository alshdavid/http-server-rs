import * as React from 'react'
import * as ReactDOM from 'react-dom/client';

const App = (props: { title: string }) => <div className="app">{props.title}</div>;

const root = ReactDOM.createRoot(document.getElementById('root')!);
root.render(<App title="Hello World" />);
