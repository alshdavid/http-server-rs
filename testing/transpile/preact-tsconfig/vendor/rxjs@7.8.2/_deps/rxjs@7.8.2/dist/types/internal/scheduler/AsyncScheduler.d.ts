import { Scheduler } from '../Scheduler.js';
import { Action } from './Action.js';
import { AsyncAction } from './AsyncAction.js';
export declare class AsyncScheduler extends Scheduler {
    actions: Array<AsyncAction<any>>;
    constructor(SchedulerAction: typeof Action, now?: () => number);
    flush(action: AsyncAction<any>): void;
}
//# sourceMappingURL=AsyncScheduler.d.ts.map