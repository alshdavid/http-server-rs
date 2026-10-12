import { h, render, type TargetedInputEvent } from "preact";
import { rx, useViewModel, reactiveArray, useAsync } from "rx";
import { BehaviorSubject } from "rxjs";

export class AppViewModel {
  @rx accessor message = "";
  @rx accessor reactiveMessages = reactiveArray<string>([]);
  asyncMessage = new BehaviorSubject<string>("");

  addRandomNumber() {
    this.reactiveMessages.push((Math.random() * 100000).toFixed(0));
  }
}

export function App() {
  const vm = useViewModel(AppViewModel, []);

  return (
    <div>
      {/* Example of reactive property assignment */}
      <h1>Reactive Message</h1>
      <p>Message: {vm.message}</p>
      <input
        onInput={(e: TargetedInputEvent<HTMLInputElement>) =>
          (vm.message = e.currentTarget.value)
        }
        value={vm.message}
      />

      <br />
      <br />

      {/* Example of update on rxjs update */}
      <h1>Rxjs Message</h1>
      <p>Message: {useAsync(vm.asyncMessage)}</p>
      <input
        onInput={(e: TargetedInputEvent<HTMLInputElement>) =>
          vm.asyncMessage.next(e.currentTarget.value)
        }
        value={useAsync(vm.asyncMessage)}
      />

      <br />
      <br />

      {/* Example of updating on array mutation */}
      <h1>Reactive Array</h1>
      <button id="adder" type="button" onClick={() => vm.addRandomNumber()}>
        Add
      </button>
      <br />
      {vm.reactiveMessages.map((m) => (
        <div key={m}>{m}</div>
      ))}
    </div>
  );
}

render(<App />, document.getElementById("root")!);
