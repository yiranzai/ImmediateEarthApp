<template>
  <div class="relative w-screen h-screen overflow-hidden bg-black">
    <!-- 加载状态 -->
    <div v-if="isLoading" class="absolute inset-0 flex items-center justify-center bg-black/70 z-10">
      <div class="text-white text-center">
        <div class="loader mb-4"></div>
        <p>Loading satellite images...</p>
      </div>
    </div>

    <!-- 错误信息 -->
    <div v-else-if="errorMessage" class="absolute inset-0 flex items-center justify-center bg-red-900/70 z-10 p-4">
      <p class="text-white text-center">{{ errorMessage }}</p>
    </div>

    <!-- 图像容器用于CSS裁剪 - 添加id="satellite-container" -->
    <div id="satellite-container" class="relative w-full h-full overflow-hidden">
      <!-- 卫星图像画布 -->
      <canvas
        id="satellite-canvas"
        ref="canvasRef"
        :style="imageStyle"
        class="absolute top-0 left-0 transition-transform duration-300 ease-in-out"
        :class="{ hidden: isLoading || errorMessage }"
      ></canvas>
    </div>

    <!-- 时间日期悬浮层 (仿macOS锁屏样式) -->
    <div class="absolute left-1/2 top-[22%] -translate-x-1/2 -translate-y-1/2 flex flex-col items-center pointer-events-none z-20">
      <!-- 日期在上 -->
      <div class="text-white/80 font-extrabold tracking-wider text-[clamp(1.2rem,1.2vw,1.7rem)] drop-shadow-[0_4px_24px_rgba(0,0,0,0.35)] select-none"
        style="font-family: 'SF Pro Display', 'SF Pro', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif;">
        {{ formatDate(currentTime, 'M月d日 EEEE') }}
      </div>
      <!-- 时间在下 -->
      <div class="text-white/80 leading-none font-extrabold tracking-wider text-[clamp(5rem,9vw,8rem)] drop-shadow-[0_4px_24px_rgba(0,0,0,0.35)] select-none"
        style="font-family: 'SF Pro Display', 'SF Pro', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Arial, sans-serif;">
        {{ formatDate(currentTime, 'HH:mm') }}
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 仅保留无法通过Tailwind实现的动画定义 */
.loader {
  width: 48px;
  height: 48px;
  border: 5px solid #fff;
  border-bottom-color: #165dff;
  border-radius: 50%;
  display: inline-block;
  box-sizing: border-box;
  animation: rotation 1s linear infinite;
}

@keyframes rotation {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

/* 响应式调整 */
@media (max-width: 768px) {
  .datetime-overlay > div {
    padding: 1rem 2rem;
  }
}

/* 图像容器样式修正 */
.relative.w-full.h-full.overflow-hidden {
  position: absolute !important;
  top: 0 !important;
  left: 0 !important;
}

/* 确保canvas正确显示 */
canvas {
  position: absolute !important;
  top: 0 !important;
  left: 0 !important;
}
</style>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useStore } from '../store'
import { formatDate } from '../services/dateUtils'

const store = useStore()
// 为了避免类型"null"不能赋值给类型"HTMLCanvasElement"的问题，将类型定义为可空类型
const canvasRef = ref<HTMLCanvasElement | null>(null)
const currentTime = ref(new Date())
const isLoading = ref(true)
const errorMessage = ref('')
const imageStyle = ref<Record<string, string>>({})

// 图像参数
const multiple = 4
const tileSize = 550
const tiles: { x: number; y: number; image: HTMLImageElement }[] = []

// 计算当前可用的卫星图像时间（每10分钟更新一次）
const getImageTime = () => {
  const now = new Date()
  now.setUTCMinutes(now.getUTCMinutes()-10) // 减去10分钟以确保数据可用
  now.setUTCMinutes(Math.floor(now.getUTCMinutes() / 10) * 10)
  return now
}

// 生成瓦片URL
const generateTileUrls = () => {
  const time = getImageTime()
  const year = time.getUTCFullYear()
  const month = String(time.getUTCMonth() + 1).padStart(2, '0')
  const day = String(time.getUTCDate()).padStart(2, '0')
  const hour = String(time.getUTCHours()).padStart(2, '0')
  const minute = String(time.getUTCMinutes()).padStart(2, '0')

  const urls: string[] = []
  for (let y = 0; y < multiple; y++) {
    for (let x = 0; x < multiple; x++) {
      urls.push(
        `https://himawari.asia/img/D531106/${multiple}d/${tileSize}/${year}/${month}/${day}/${hour}${minute}00_${x}_${y}.png`
      )
    }
  }
  return urls
}

// 加载图像瓦片
const loadTiles = async () => {
  isLoading.value = true
  errorMessage.value = ''
  tiles.length = 0

  try {
    const urls = generateTileUrls()
    const promises = urls.map((url, index) => {
      return new Promise<{ x: number; y: number; image: HTMLImageElement }>((resolve, reject) => {
        const img = new Image()
        img.src = url
        img.onload = () => {
          resolve({
            x: index % multiple,
            y: Math.floor(index / multiple),
            image: img,
          })
        }
        img.onerror = () => {
          const error = new Error(`Failed to load tile: ${url}`)
          console.error(`[${new Date().toISOString()}] Error loading tile: ${url}`, error)
          reject(error)
        }
      })
    })

    const loadedTiles = await Promise.all(promises)
    loadedTiles.forEach(tile => tiles.push(tile))
    renderImage()
  } catch (err) {
    const errorMsg = err instanceof Error ? err.message : 'Failed to load satellite images'
    errorMessage.value = errorMsg
    console.error(`[${new Date().toISOString()}] Satellite image loading failed:`, err)
  } finally {
    isLoading.value = false
  }
}

// 渲染拼接后的图像并应用智能裁剪
const renderImage = () => {
  if (!canvasRef.value || tiles.length === 0) return

  const canvas = canvasRef.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // 设置画布大小
  canvas.width = tileSize * multiple
  canvas.height = tileSize * multiple

  // 绘制所有瓦片
  tiles.forEach(tile => {
    ctx.drawImage(tile.image, tile.x * tileSize, tile.y * tileSize, tileSize, tileSize)
  })

  applySmartCropping()
}

// 应用智能裁剪算法 (添加详细日志)
const applySmartCropping = () => {
  console.log('[applySmartCropping] 开始执行智能裁剪逻辑')
  const now = new Date()
  const japanTime = new Date(now.toLocaleString('en-US', { timeZone: 'Asia/Tokyo' }))
  const hours = japanTime.getHours()
  const isAfternoon = hours >= 15
  const isEarlyMorning = hours < 6
  console.log(
    `[applySmartCropping] 日本时间: ${japanTime.toLocaleString()}, 小时: ${hours}, 下午: ${isAfternoon}, 凌晨: ${isEarlyMorning}`
  )

  // 获取容器和canvas元素
  const container = document.getElementById('satellite-container')
  const canvas = canvasRef.value
  console.log(
    `[applySmartCropping] 容器元素: ${container ? '找到' : '未找到'}, Canvas元素: ${canvas ? '找到' : '未找到'}`
  )

  if (!container || !canvas) {
    console.error('[applySmartCropping] 容器或Canvas元素未找到，无法应用裁剪')
    return
  }

  // 获取尺寸信息
  const containerWidth = container.clientWidth
  const containerHeight = container.clientHeight
  const imgNaturalWidth = canvas.width
  const imgNaturalHeight = canvas.height
  console.log(
    `[applySmartCropping] 容器尺寸: ${containerWidth}x${containerHeight}, 图像尺寸: ${imgNaturalWidth}x${imgNaturalHeight}`
  )

  // 判断屏幕方向（横屏/竖屏）
  const isPortrait = containerHeight > containerWidth
  console.log(`[applySmartCropping] 屏幕方向: ${isPortrait ? '竖屏' : '横屏'}`)

  // 计算缩放比例 - 修复竖屏逻辑
  let scale, imgScaledWidth, imgScaledHeight
  if (isPortrait) {
    // 竖屏: 优先按高度缩放，确保高度充满屏幕
    scale = containerHeight / imgNaturalHeight
    imgScaledHeight = containerHeight
    imgScaledWidth = imgNaturalWidth * scale
  } else {
    // 横屏: 优先按宽度缩放，确保宽度充满屏幕
    scale = containerWidth / imgNaturalWidth
    imgScaledWidth = containerWidth
    imgScaledHeight = imgNaturalHeight * scale
  }
  console.log(
    `[applySmartCropping] 缩放比例: ${scale.toFixed(4)}, 缩放后图像尺寸: ${Math.round(imgScaledWidth)}x${Math.round(imgScaledHeight)}`
  )

  // 计算黑边宽度
  const blackBorderSize = Math.round(imgScaledWidth / 12)
  console.log(`[applySmartCropping] 黑边宽度: ${blackBorderSize}px`)

  // 计算偏移量
  let offsetX = 0
  let offsetY = 0

  // 水平方向偏移（左右裁剪）- 仅当图片宽度大于容器宽度时
  if (imgScaledWidth > containerWidth) {
    const overflowWidth = imgScaledWidth - containerWidth
    if (isEarlyMorning) {
      offsetX = -overflowWidth // 凌晨显示右侧
    } else {
      offsetX = 0 + blackBorderSize + blackBorderSize // 其他时间
      offsetY = blackBorderSize
    }
    console.log(
      `[applySmartCropping] 水平裁剪 - 溢出宽度: ${Math.round(overflowWidth)}px, 偏移X: ${Math.round(offsetX)}px`
    )
  }

  // 垂直方向偏移（上下裁剪）- 仅当图片高度大于容器高度时
  if (imgScaledHeight > containerHeight) {
    let overflowHeight = imgScaledHeight - containerHeight
    let overflowWidth = imgScaledWidth - containerWidth
    if (isAfternoon) {
      offsetX = 0 + overflowWidth + blackBorderSize // 下午显示左侧
      offsetY = blackBorderSize
    } else if (isEarlyMorning) {
      offsetX = -overflowWidth - blackBorderSize // 凌晨显示右侧
      offsetY = -blackBorderSize
    } else {
      // 中间显示, 缩小图片
      imgScaledWidth = imgScaledWidth * 0.9
      imgScaledHeight = imgScaledHeight * 0.9
      overflowWidth = imgScaledWidth - containerWidth
      offsetX = -overflowWidth / 2
      offsetY = -overflowWidth / 2
    } // 始终显示顶部
    console.log(`[applySmartCropping] 垂直裁剪 - 溢出高度: ${Math.round(overflowHeight)}px, 偏移Y: ${offsetY}px`)
  }

  // 应用样式
  canvas.style.width = `${imgScaledWidth}px`
  canvas.style.height = `${imgScaledHeight}px`
  canvas.style.transform = `translate(${offsetX}px, ${offsetY}px)`
  console.log(
    `[applySmartCropping] 应用样式 - width: ${imgScaledWidth}px, height: ${imgScaledHeight}px, transform: translate(${offsetX}px, ${offsetY}px)`
  )
}

// 定时更新图像和时间
const startUpdates = () => {
  // 每分钟更新一次时间
  setInterval(() => {
    currentTime.value = new Date()
  }, 60000)

  // 每10分钟尝试更新一次图像
  loadTiles()
  setInterval(loadTiles, 10 * 60 * 1000)
}

// 添加防抖函数
function debounce<T extends (...args: any[]) => void>(func: T, delay: number): T {
  let timeoutId: number | null = null
  return function (this: any, ...args: Parameters<T>) {
    if (timeoutId !== null) {
      clearTimeout(timeoutId)
    }
    timeoutId = window.setTimeout(() => {
      func.apply(this, args)
    }, delay)
  } as T
}

onMounted(() => {
  startUpdates()
  // 添加窗口大小调整事件监听 - 添加日志
  const handleResize = debounce(() => {
    console.log('[resize] 窗口大小变化，触发裁剪更新')
    if (!isLoading.value && !errorMessage.value) {
      applySmartCropping()
    }
  }, 100)
  window.addEventListener('resize', handleResize)
  console.log('[onMounted] 已添加窗口大小调整事件监听')

  // 初始加载完成后触发一次裁剪
  setTimeout(() => {
    console.log('[onMounted] 初始加载完成，触发初始裁剪')
    applySmartCropping()
  }, 1000)

  // 组件卸载时移除事件监听
  onUnmounted(() => {
    window.removeEventListener('resize', handleResize)
    console.log('[onUnmounted] 已移除窗口大小调整事件监听')
  })
})
</script>

<style scoped>
#satellite-container {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden !important;
}

#satellite-image {
  position: absolute;
  top: 0;
  left: 0;
  transition: transform 0.3s ease;
}
</style>
