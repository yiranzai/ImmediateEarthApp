export function formatDate(date: Date, format: string): string {
  if (format === 'HH:mm') {
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');

    return `${hours}:${minutes}`;
  }

  if (format === 'yyyy-MM-dd EEEE') {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    const weekday = date.toLocaleDateString('zh-CN', { weekday: 'long' });
    return `${year}-${month}-${day} ${weekday}`;
  }

  return date.toISOString();
}
