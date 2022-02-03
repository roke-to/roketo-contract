export const formatDistanceLocale = {
  xYears: '{{count}} years',
  xMonths: '{{count}} months',
  xDays: '{{count}}d',
  xSeconds: '{{count}}s',
  xMinutes: '{{count}}m',
  xHours: '{{count}}h',
};

export const shortEnLocale = {
  formatDistance: (token, count) => {
    return formatDistanceLocale[token].replace('{{count}}', count);
  },
};

export function isValidDate(d) {
  return d instanceof Date && !isNaN(d);
}