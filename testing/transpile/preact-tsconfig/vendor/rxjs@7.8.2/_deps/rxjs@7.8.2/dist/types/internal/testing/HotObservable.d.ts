import { Subject } from '../Subject.js';
import { Scheduler } from '../Scheduler.js';
import { TestMessage } from './TestMessage.js';
import { SubscriptionLog } from './SubscriptionLog.js';
import { SubscriptionLoggable } from './SubscriptionLoggable.js';
export declare class HotObservable<T> extends Subject<T> implements SubscriptionLoggable {
    messages: TestMessage[];
    subscriptions: SubscriptionLog[];
    scheduler: Scheduler;
    logSubscribedFrame: () => number;
    logUnsubscribedFrame: (index: number) => void;
    constructor(messages: TestMessage[], scheduler: Scheduler);
    setup(): void;
}
//# sourceMappingURL=HotObservable.d.ts.map