import { Observable } from '../Observable.js';
import { ObservableInputTuple } from '../types.js';
export declare function onErrorResumeNext<A extends readonly unknown[]>(sources: [...ObservableInputTuple<A>]): Observable<A[number]>;
export declare function onErrorResumeNext<A extends readonly unknown[]>(...sources: [...ObservableInputTuple<A>]): Observable<A[number]>;
//# sourceMappingURL=onErrorResumeNext.d.ts.map