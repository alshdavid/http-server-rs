import { Observable } from '../Observable.js';
import { ObservableInput, OperatorFunction, ObservedValueOf } from '../types.js';
export declare function catchError<T, O extends ObservableInput<any>>(selector: (err: any, caught: Observable<T>) => O): OperatorFunction<T, T | ObservedValueOf<O>>;
//# sourceMappingURL=catchError.d.ts.map