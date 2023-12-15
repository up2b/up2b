/**
 * 通过扩展名筛选图片
 * */
export const filterImages = (
  filePaths: string[],
  allowedFormats: AllowedImageFormat[],
): string[] => {
  return filePaths.filter((filePath) =>
    allowedFormats.some((ext) =>
      ext === 'JPEG'
        ? filePath.toLowerCase().endsWith('.jpg') ||
          filePath.toLowerCase().endsWith('.jpeg')
        : filePath.toUpperCase().endsWith('.' + ext),
    ),
  )
}
