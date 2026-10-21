import { type ComponentChild, type ComponentType, type ContainerNode } from "./_deps/preact@11.0.1/src/index.js";
type Callback = () => unknown;
export declare class Reactive {
    #private;
    next(): void;
    subscribe(callback: Callback): () => void;
    addChild(child: Reactive | undefined): void;
    removeChild(child: Reactive | undefined): void;
}
export declare function rx<T extends object, V>(target: ClassAccessorDecoratorTarget<T, V>, context: ClassAccessorDecoratorContext<T, V>): {
    get(this: T): V;
    set(this: T, value: V): void;
    init(this: T, initialValue: V): V;
};
/** Creates a reactive proxy over an array. Mutations notify subscribers. */
export declare function reactiveArray<T>(values?: T[]): T[];
/** Creates a reactive proxy over a plain object. Mutations notify subscribers. */
export declare function reactiveObject<T extends object>(values: T): T;
export declare function subscribe(target: object, callback: Callback): () => void;
export declare function subscribe(target: object, property: PropertyKey, callback: Callback): () => void;
export interface ViewModelLifecycle {
    onInit?(): void | Promise<void>;
    onDestroy?(): void | Promise<void>;
}
export declare function useViewModel<T extends object, U extends Array<unknown>>(ctor: new (...args: U) => T, args: U): T;
export declare function useReactive<T extends object>(instance: T): T;
export declare const InjectContext: import("./_deps/preact@11.0.1/src/index.js").Context<Provider>;
export declare class Provider extends Map<any, any> {
    static Provider: import("./_deps/preact@11.0.1/src/index.js").Provider<Provider>;
    provide(token: any, value: any): void;
}
export declare function useInjectContext(): Provider;
export declare function useInject<C extends new (...args: any[]) => any>(key: C): InstanceType<C>;
export declare function useInject<T>(key: any): T;
interface Unsubscribable {
    unsubscribe(): void;
}
interface Observer<T> {
    next: (value: T) => void;
}
interface Subscribable<T> {
    subscribe(observer: Observer<T>): Unsubscribable;
}
interface BehaviorLike<T> extends Subscribable<T> {
    getValue(): T;
}
export declare function useAsync<T>(x: BehaviorLike<T>): T;
export declare function useAsync<T>(x: Subscribable<T>, defaultValue: T): T;
type Query = Record<string, string>;
type Handler = (query: Query) => unknown | Promise<unknown>;
export declare class Router {
    #private;
    path(pattern: string, handler: Handler): void;
    bootstrap(): {
        disconnect: () => void;
    };
}
export type PreactAppOptions = {
    root?: ContainerNode | null;
};
export declare class PreactApp {
    #private;
    constructor({ root }: PreactAppOptions);
    use<Props extends {
        value: unknown;
        children?: ComponentChild;
    }>(contextProvider: ComponentType<Props>, value: Props["value"]): void;
    mount(Component: ComponentChild | ComponentType<Record<PropertyKey, never>>): void;
}
export declare const ROUTER_PARAMS: unique symbol;
export type MountContext = {
    mount: (Component: ComponentChild | ComponentType<Record<PropertyKey, never>>) => void;
    unmount: () => void;
};
export type PreactRouterOptions = {
    root?: ContainerNode | null;
};
export declare class PreactRouter {
    #private;
    constructor({ root }: PreactRouterOptions);
    inject(key: any, value: any): void;
    path(path: string, handler: (ctx: MountContext) => (any | Promise<any>)): void;
    bootstrap(): void;
}
export declare const useRouteParams: () => Record<string, string>;
export {};
