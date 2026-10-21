import { Scheduler } from '../Scheduler.js';
import { SubscriptionLog } from './SubscriptionLog.js';
export declare class SubscriptionLoggable {
    subscriptions: SubscriptionLog[];
    scheduler: Scheduler;
    logSubscribedFrame(): number;
    logUnsubscribedFrame(index: number): void;
}
//# sourceMappingURL=SubscriptionLoggable.d.ts.map