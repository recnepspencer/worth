import type {
  CallableSignals,
  ComputedSignalHandle,
  FormController,
  InputSignalHandle,
  OutputSignalHandle,
} from "./index.js";

declare const signals: CallableSignals;
declare const form: FormController<{ title: string }>;
const summary = form.summarySignal();

signals.effect(summary, () => {});
signals.watch(summary, notice => { void notice.meaningfulChange; });
signals.read<ReturnType<typeof summary>>(summary);
signals.effect(summary.id, () => {});
signals.watch(summary.id, () => {});

type Metadata = { readonly labels: readonly string[] };
declare const input: InputSignalHandle<Metadata>;
declare const computed: ComputedSignalHandle<Metadata>;
declare const output: OutputSignalHandle<Metadata>;
for (const target of [input, computed, output]) {
  signals.effect(target, () => {});
  signals.watch(target, () => {});
}

// @ts-expect-error observation targets must retain branded signal identity
signals.effect(() => summary(), () => {});
// @ts-expect-error an object with an id is not a branded signal handle
signals.watch({ id: summary.id }, () => {});
// @ts-expect-error arbitrary values are not observation targets
signals.effect(42, () => {});

declare const count: InputSignalHandle<number>;
count.set(1);
// @ts-expect-error accepting metadata observation must not weaken input writes
count.set("wrong value type");
