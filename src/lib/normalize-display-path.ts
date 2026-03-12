export const normalizeDisplayPath = (value: string) => {
  if (value.startsWith('\\\\?\\UNC\\')) {
    return `\\\\${value.slice('\\\\?\\UNC\\'.length)}`
  }

  if (value.startsWith('\\\\?\\')) {
    return value.slice('\\\\?\\'.length)
  }

  return value
}
