import { h } from 'preact'
import { rx, useViewModel } from './rx.ts'

export class AboutViewModel {
  @rx accessor message = "";
}

export function AboutPage() {
  const vm = useViewModel(AboutViewModel, []);

  return (
    <div>
      <nav>
        <a href="#home">Home</a>
        <a href="#about">About</a>
      </nav>
      <h1>About Page</h1>
      <p>Message: {vm.message}</p>
      <input
        onInput={(e: any) => (vm.message = e.target.value)}
        value={vm.message}
      />
    </div>
  );
}
