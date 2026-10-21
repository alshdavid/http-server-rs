import { Observable } from '../Observable.js';
import { Scheduler } from '../Scheduler.js';
import { TestMessage } from './TestMessage.js';
import { SubscriptionLog } from './SubscriptionLog.js';
import { SubscriptionLoggable } from './SubscriptionLoggable.js';
import { Subscriber } from '../Subscriber.js';
export declare class ColdObservable<T> extends Observable<T> implements SubscriptionLoggable {
    messages: TestMessage[];
    subscriptions: SubscriptionLog[];
    scheduler: Scheduler;
    logSubscribedFrame: () => number;
    logUnsubscribedFrame: (index: number) => void;
    constructor(messages: TestMessage[], scheduler: Scheduler);
    scheduleMessages(subscriber: Subscriber<any>): void;
}
//# sourceMappingURL=ColdObservable.d.ts.map