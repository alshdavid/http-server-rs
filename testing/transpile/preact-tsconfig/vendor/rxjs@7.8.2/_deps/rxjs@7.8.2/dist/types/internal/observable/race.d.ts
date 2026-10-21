import { Observable } from '../Observable.js';
import { ObservableInput, ObservableInputTuple } from '../types.js';
import { Subscriber } from '../Subscriber.js';
export declare function race<T extends readonly unknown[]>(inputs: [...ObservableInputTuple<T>]): Observable<T[number]>;
export declare function race<T extends readonly unknown[]>(...inputs: [...ObservableInputTuple<T>]): Observable<T[number]>;
/**
 * An observable initializer function for both the static version and the
 * operator version of race.
 * @param sources The sources to race
 */
export declare function raceInit<T>(sources: ObservableInput<T>[]): (subscriber: Subscriber<T>) => void;
//# sourceMappingURL=race.d.ts.map