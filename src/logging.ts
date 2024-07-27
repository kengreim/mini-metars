export function logIfDev(message?: any, ...optionalParams: any[]) {
  if (import.meta.env.DEV) {
    console.log(message, ...optionalParams);
  }
}
