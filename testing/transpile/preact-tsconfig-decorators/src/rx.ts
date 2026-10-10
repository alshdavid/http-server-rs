import { useEffect, useMemo, useReducer } from "preact/hooks";

const ON_CHANGE = Symbol("ON_CHANGE");

export class Reactive extends EventTarget {
  next() {
    super.dispatchEvent(new Event("change"));
  }

  subscribe(callback: () => any | Promise<any>): () => void {
    super.addEventListener("change", callback);
    return () => super.removeEventListener("change", callback);
  }

  static notifyChange(self: Reactive) {
    self.next();
  }
}

export function rx<T, V>(
  _target: ClassAccessorDecoratorTarget<T, V>,
  context: ClassAccessorDecoratorContext<T, V>,
) {
  const propertyKey = context.name as string;

  return {
    get(this: T): V {
      return getOrInit(this)[1][propertyKey];
    },
    set(this: T, value: V) {
      const [subject, stash] = getOrInit(this);
      if (stash[propertyKey] === value) {
        return;
      }

      stash[propertyKey] = value;
      subject.next();
    },
    init(this: T, initialValue: V): V {
      const [, stash] = getOrInit(this);
      stash[propertyKey] = initialValue;
      return initialValue;
    },
  };
}

export type RxState = [Reactive, Record<any, any>];

function getOrInit(self: any): RxState {
  if (!self[ON_CHANGE]) {
    Object.defineProperty(self, ON_CHANGE, {
      value: [new Reactive(), {}],
      configurable: false,
      writable: false,
      enumerable: false,
    });
  }
  return self[ON_CHANGE] as any;
}


export function subscribe(
  target: any,
  callback: () => any | Promise<any>,
): () => void {
  const [subject, stash] = getOrInit(target);
  const subscriptions = [subject.subscribe(callback)];

  for (const value of Object.values(stash) || []) {
    if (value instanceof Reactive) {
      subscriptions.push(value.subscribe(callback));
      continue;
    }

    if (typeof value === "object" && ON_CHANGE in value) {
      subscriptions.push(subscribe(value, callback));
    }
  }

  return () => subscriptions.forEach((cb) => cb());
}


export interface ViewModelLifecycle {
  onInit?(): any | Promise<any>;
  onDestroy?(): any | Promise<any>;
}

export function useViewModel<
  T extends ViewModelLifecycle & Object,
  U extends Array<any>,
>(ctor: new (...args: U) => T, args: U): T {
  const forceUpdate = useReducer(() => ({}), {})[1] as () => void;

  const instance = useMemo(() => new ctor(...args), [ctor, ...args]);

  useEffect(() => {
    const onChange = () => forceUpdate();
    const unsubscribe = subscribe(instance, onChange);
    instance.onInit?.();
    return () => {
      instance.onDestroy?.();
      unsubscribe();
    };
  }, [instance]);

  return instance;
}

export function useReactive<T>(instance: T): T {
  const forceUpdate = useReducer(() => ({}), {})[1] as () => void;

  useEffect(() => {
    const onChange = () => forceUpdate();
    const unsubscribe = subscribe(instance, onChange);
    return () => {
      unsubscribe();
    };
  }, [instance]);

  return instance;
}
