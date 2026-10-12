import type { CalendarEvent, ImportedEvent } from '../module_bindings/types';
import { localDate } from './dates';

export type DisplayEvent = Omit<CalendarEvent, 'id'> & { id: bigint | string; sourceName?: string };

export function importedEvent(event: ImportedEvent): DisplayEvent {
  return {
    ...event,
    // An all-day event belongs to the same calendar dates on every screen.
    startAt: event.allDay ? BigInt(localDate(event.startsOn).getTime()) : event.startAt,
    endAt: event.allDay ? BigInt(localDate(event.endsOn).getTime()) : event.endAt,
  };
}
