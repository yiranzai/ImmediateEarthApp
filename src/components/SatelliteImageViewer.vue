<template>
  <div class="relative w-screen h-screen overflow-hidden bg-black">
    <!-- 刷新按钮（左上角悬浮） -->
    <button
      class="absolute left-4 top-4 z-30 bg-black/60 hover:bg-black/80 text-white rounded-full p-2 shadow-lg transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-400"
      :disabled="isLoading"
      @click="onRefresh"
      aria-label="刷新卫星图像"
      title="刷新卫星图像"
    >
      <!-- 刷新SVG图标 -->
      <svg
        v-if="!isLoading"
        xmlns="http://www.w3.org/2000/svg"
        class="h-6 w-6"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M4 4v5h.582m15.356-2.582A9 9 0 11 5.582 9M4 9V4m0 0h5"
        />
      </svg>
      <!-- loading 状态下显示旋转动画 -->
      <svg v-else class="animate-spin h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"></path>
      </svg>
    </button>
    <!-- 设置按钮（右上角悬浮） -->
    <button
      class="absolute right-4 top-4 z-30 bg-black/60 hover:bg-black/80 text-white rounded-full p-2 shadow-lg transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-400"
      @click="openSettings"
      aria-label="设置"
      title="设置"
    >
      <!-- 齿轮SVG图标 -->
      <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M11.049 2.927c.3-1.14 1.603-1.14 1.902 0a1.724 1.724 0 002.573 1.01c.943-.545 2.042.454 1.497 1.398a1.724 1.724 0 001.01 2.573c1.14.3 1.14 1.603 0 1.902a1.724 1.724 0 00-1.01 2.573c.545.943-.454 2.042-1.398 1.497a1.724 1.724 0 00-2.573 1.01c-.3 1.14-1.603 1.14-1.902 0a1.724 1.724 0 00-2.573-1.01c-.943.545-2.042-.454-1.497-1.398a1.724 1.724 0 00-1.01-2.573c-1.14-.3-1.14-1.603 0-1.902a1.724 1.724 0 001.01-2.573c-.545-.943.454-2.042 1.398-1.497.943.545 2.042-.454 1.497-1.398z"
        />
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </svg>
    </button>
    <!-- 设置弹窗 -->
    <div v-if="showSettings" class="fixed inset-0 z-40 flex items-center justify-center bg-black/40">
      <div class="bg-white rounded-2xl shadow-2xl p-8 w-[90vw] max-w-md relative">
        <h2 class="text-2xl font-bold mb-6 text-black">设置</h2>
        <form @submit.prevent="saveSettings">
          <div class="mb-6">
            <label class="block mb-2 text-black font-semibold">定时加载间隔（分钟）</label>
            <input
              type="number"
              min="1"
              v-model.number="settings.interval"
              class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-400 bg-white text-black text-lg transition"
            />
          </div>
          <div class="mb-6">
            <label class="block mb-2 text-black font-semibold">OpenWeatherMap Key</label>
            <input
              type="text"
              v-model="settings.owmKey"
              class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-400 bg-white text-black text-lg transition"
            />
          </div>
          <div class="mb-8">
            <label class="block mb-2 text-black font-semibold">OpenWeatherMap 地址</label>
            <input
              type="text"
              v-model="settings.owmUrl"
              class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-400 bg-white text-black text-lg transition"
            />
          </div>
          <div class="flex justify-end gap-4">
            <button
              type="button"
              class="px-5 py-2 rounded-lg bg-gray-200 text-gray-700 hover:bg-gray-300 font-semibold"
              @click="showSettings = false"
            >
              取消
            </button>
            <button
              type="submit"
              class="px-5 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700 font-semibold shadow transition"
            >
              保存
            </button>
          </div>
        </form>
      </div>
    </div>

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
    <div
      id="satellite-container"
      class="relative w-full h-full overflow-hidden"
      @dblclick="onSaveImage"
      @touchstart="onTouchStart"
      @touchend="onTouchEnd"
    >
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
    <div
      class="absolute left-1/2 top-[22%] -translate-x-1/2 -translate-y-1/2 flex flex-col items-center pointer-events-none z-20"
    >
      <!-- 日期在上 -->
      <div
        class="text-white/80 font-extrabold tracking-wider text-[clamp(1.2rem,1.2vw,1.7rem)] drop-shadow-[0_4px_24px_rgba(0,0,0,0.35)] select-none"
        style="
          font-family:
            'SF Pro Display',
            'SF Pro',
            -apple-system,
            BlinkMacSystemFont,
            'Segoe UI',
            Roboto,
            Arial,
            sans-serif;
        "
      >
        {{ formatDate(currentTime, 'M月d日 EEEE') }}
      </div>
      <!-- 时间在下 -->
      <div
        class="text-white/80 leading-none font-extrabold tracking-wider text-[clamp(5rem,9vw,8rem)] drop-shadow-[0_4px_24px_rgba(0,0,0,0.35)] select-none"
        style="
          font-family:
            'SF Pro Display',
            'SF Pro',
            -apple-system,
            BlinkMacSystemFont,
            'Segoe UI',
            Roboto,
            Arial,
            sans-serif;
        "
      >
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
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useStore } from '../store'
import { formatDate } from '../services/dateUtils'
import { invoke } from '@tauri-apps/api/core'
import { Store } from '@tauri-apps/plugin-store'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile, BaseDirectory } from '@tauri-apps/plugin-fs'
import { info, warn, error, debug, trace, attachConsole } from '@tauri-apps/plugin-log'
// import { request, check, Permission } from '@tauri-apps/plugin-permission'
function forwardConsole(
  fnName: 'log' | 'debug' | 'info' | 'warn' | 'error',
  logger: (message: string) => Promise<void>
) {
  const original = console[fnName]
  console[fnName] = message => {
    original(message)
    logger(message)
  }
}

forwardConsole('log', trace)
forwardConsole('debug', debug)
forwardConsole('info', info)
forwardConsole('warn', warn)
forwardConsole('error', error)

const store = useStore()
// 为了避免类型"null"不能赋值给类型"HTMLCanvasElement"的问题，将类型定义为可空类型
const canvasRef = ref<HTMLCanvasElement | null>(null)
const currentTime = ref(new Date())
const isLoading = ref(true)
const errorMessage = ref('')
const imageStyle = ref<Record<string, string>>({})
const hasLoadedImage = ref(false) // 是否已成功加载过图片

// 图像参数
const multiple = 4
const tileSize = 550
const tiles: { x: number; y: number; image: HTMLImageElement }[] = []

// 设置弹窗显示状态
const showSettings = ref(false)
// 设置项，初始化时从 localStorage 读取
const settings = ref({
  interval: Number(localStorage.getItem('satellite_interval')) || 10,
  owmKey: localStorage.getItem('owm_key') || '',
  owmUrl: localStorage.getItem('owm_url') || '',
})

// 在模块作用域只初始化一次
let storePromise: Promise<Store> | null = null

function getStore() {
  console.info('[getStore] 进入函数')
  if (!storePromise) {
    storePromise = Store.load('settings.json')
  }
  return storePromise
}

// 计算当前可用的卫星图像时间（每10分钟更新一次）
const getImageTime = () => {
  const now = new Date()
  now.setUTCMinutes(now.getUTCMinutes()) // 减去10分钟以确保数据可用
  now.setUTCMinutes(Math.floor(now.getUTCMinutes() / 10) * 10)
  return now
}

// 生成瓦片URL，支持传入指定时间
const generateTileUrls = (timeArg?: Date) => {
  console.info('[generateTileUrls] 进入函数')
  const time = timeArg ? new Date(timeArg) : getImageTime()
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

// 加载并拼接卫星图像（支持递归回退时间）
async function loadStitchedImage(timeArg?: Date, retryCount = 0) {
  console.info('[loadStitchedImage] 进入函数')
  // 最大回退次数，防止死循环
  const MAX_RETRY = 12
  // 只有首次未加载过图片时才 loading
  if (!hasLoadedImage.value) {
    isLoading.value = true
  }
  errorMessage.value = ''
  try {
    // 生成瓦片 URL 数组
    const urls = generateTileUrls(timeArg)
    // 调用后端方法获取拼接后的 base64 图像
    const base64 = await invoke<string>('fetch_and_stitch_tiles', {
      urls,
      tilesPerRow: multiple,
      tileSize,
    })
    // 创建图片对象
    const img = new window.Image()
    img.src = 'data:image/png;base64,' + base64
    // 图片加载完成后绘制到 canvas
    img.onload = () => {
      if (!canvasRef.value) return
      const canvas = canvasRef.value
      const ctx = canvas.getContext('2d')
      if (!ctx) return
      canvas.width = img.width
      canvas.height = img.height
      ctx.clearRect(0, 0, canvas.width, canvas.height)
      ctx.drawImage(img, 0, 0)
      applySmartCropping()
      hasLoadedImage.value = true
      isLoading.value = false
    }
  } catch (err) {
    // 如果是无效瓦片数据，首次加载失败才递归回退时间
    if (err && typeof err === 'string' && err.includes('无效瓦片数据')) {
      console.warn(`[loadStitchedImage] 检测到无效瓦片数据，本次不渲染，等待下次自动重试，当前重试次数: ${retryCount}`)
      // 只有首次未加载过图片时才递归回退时间
      if (!hasLoadedImage.value && retryCount < MAX_RETRY) {
        // 往前推 60 分钟
        const prevTime = timeArg ? new Date(timeArg) : getImageTime()
        prevTime.setUTCMinutes(prevTime.getUTCMinutes() - 60)
        // 递归重试
        await loadStitchedImage(prevTime, retryCount + 1)
        return
      }
      // 超过最大重试次数，显示错误
      if (!hasLoadedImage.value && retryCount >= MAX_RETRY) {
        errorMessage.value = '长时间未获取到有效卫星图像，请稍后重试'
        isLoading.value = false
      } else if (!hasLoadedImage.value) {
        isLoading.value = true
      } else {
        isLoading.value = false
      }
      return
    }
    const errorMsg = err instanceof Error ? err.message : '卫星图像加载失败'
    errorMessage.value = errorMsg
    console.error(`[${new Date().toISOString()}] 卫星图像加载失败: ${err}`)
  } finally {
    // 只有首次未加载过图片且没有 errorMessage 时才关闭 loading
    if (!errorMessage.value && hasLoadedImage.value) {
      isLoading.value = false
    }
  }
}

// 应用智能裁剪算法 (添加详细日志)
const applySmartCropping = () => {
  console.info('[applySmartCropping] 进入函数')
  const now = new Date()
  const japanTime = new Date(now.toLocaleString('en-US', { timeZone: 'Asia/Tokyo' }))
  const hours = japanTime.getHours()
  const isAfternoon = hours >= 15
  const isEarlyMorning = hours < 6
  console.warn(
    `[applySmartCropping] 日本时间: ${japanTime.toLocaleString()}, 小时: ${hours}, 下午: ${isAfternoon}, 凌晨: ${isEarlyMorning}`
  )

  // 获取容器和canvas元素
  const container = document.getElementById('satellite-container')
  const canvas = canvasRef.value
  info(`[applySmartCropping] 容器元素: ${container ? '找到' : '未找到'}, Canvas元素: ${canvas ? '找到' : '未找到'}`)
  // 日志：打印设备屏幕尺寸、窗口尺寸、容器尺寸
  info(
    `[applySmartCropping] 设备屏幕尺寸: ${window.screen.width}x${window.screen.height}，窗口尺寸: ${window.innerWidth}x${window.innerHeight}，容器尺寸: ${container ? container.clientWidth + 'x' + container.clientHeight : '未知'}`
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
  console.info(
    `[applySmartCropping] 容器尺寸: ${containerWidth}x${containerHeight}, 图像尺寸: ${imgNaturalWidth}x${imgNaturalHeight}`
  )

  // 判断屏幕方向（横屏/竖屏）
  const isPortrait = containerHeight > containerWidth
  console.info(`[applySmartCropping] 屏幕方向: ${isPortrait ? '竖屏' : '横屏'}`)

  // 计算缩放比例 - 修复竖屏逻辑
  let scale, imgScaledWidth, imgScaledHeight, blackBorderXSize, blackBorderYSize
  if (isPortrait) {
    // 竖屏: 优先按高度缩放，确保高度充满屏幕
    scale = containerHeight / imgNaturalHeight
    imgScaledHeight = containerHeight
    imgScaledWidth = imgNaturalWidth * scale
    blackBorderXSize = Math.round(containerWidth / 12)
    blackBorderYSize = Math.round(containerHeight / 12)
  } else {
    // 横屏: 优先按宽度缩放，确保宽度充满屏幕
    scale = containerWidth / imgNaturalWidth
    imgScaledWidth = containerWidth
    imgScaledHeight = imgNaturalHeight * scale
    blackBorderXSize = Math.round(containerWidth / 12)
    blackBorderYSize = Math.round(containerHeight / 12)
  }
  console.info(
    `[applySmartCropping] 缩放比例: ${scale.toFixed(4)}, 缩放后图像尺寸: ${Math.round(imgScaledWidth)}x${Math.round(imgScaledHeight)} `
  )

  // 计算黑边宽度
  
  console.info(`[applySmartCropping] 黑边宽度: ${blackBorderXSize}px, ${blackBorderYSize}px`)

  // 计算偏移量
  let offsetX = 0
  let offsetY = 0

  // 水平方向偏移（左右裁剪）- 仅当图片宽度大于容器宽度时
  if (imgScaledWidth > containerWidth) {
    const overflowWidth = imgScaledWidth - containerWidth
    if (isEarlyMorning) {
      offsetX = -overflowWidth // 凌晨显示右侧
    } else {
      offsetX = 0 + blackBorderXSize // 其他时间
      offsetY = blackBorderYSize
    }
    console.info(
      `[applySmartCropping] 水平裁剪 - 溢出宽度: ${Math.round(overflowWidth)}px, 偏移X: ${Math.round(offsetX)}px`
    )
  }

  // 垂直方向偏移（上下裁剪）- 仅当图片高度大于容器高度时
  if (imgScaledHeight > containerHeight) {
    let overflowHeight = imgScaledHeight - containerHeight
    let overflowWidth = imgScaledWidth - containerWidth
    if (isAfternoon) {
      offsetX = 0 + overflowWidth + blackBorderXSize // 下午显示左侧
      offsetY = blackBorderYSize
    } else if (isEarlyMorning) {
      offsetX = -overflowWidth - blackBorderXSize // 凌晨显示右侧
      offsetY = -blackBorderYSize
    } else {
      // 中间显示, 缩小图片
      imgScaledWidth = imgScaledWidth * 0.9
      imgScaledHeight = imgScaledHeight * 0.9
      overflowWidth = imgScaledWidth - containerWidth
      offsetX = -overflowWidth / 2
      offsetY = -overflowWidth / 2
    } // 始终显示顶部
    console.info(`[applySmartCropping] 垂直裁剪 - 溢出高度: ${Math.round(overflowHeight)}px, 偏移Y: ${offsetY}px`)
  }
  canvas.style.width = `${imgScaledWidth}px`
  canvas.style.height = `${imgScaledHeight}px`
  canvas.style.transform = `translate(${offsetX}px, ${offsetY}px)`
  console.info(
    `[applySmartCropping] 应用样式 - width: ${imgScaledWidth}px, height: ${imgScaledHeight}px, transform: translate(${offsetX}px, ${offsetY}px)`
  )
}

// 保存设置项到 localStorage，并关闭弹窗
async function saveSettings() {
  console.info('[saveSettings] 进入函数')
  const store = await getStore()
  await store.set('interval', settings.value.interval)
  await store.set('owmKey', settings.value.owmKey)
  await store.set('owmUrl', settings.value.owmUrl)
  await store.save()
  showSettings.value = false
  resetIntervalTimer()
}

// 定时器句柄
let intervalTimer: number | null = null

// 重置定时器，使用新的间隔
function resetIntervalTimer() {
  console.info('[resetIntervalTimer] 进入函数')
  if (intervalTimer !== null) {
    clearInterval(intervalTimer)
  }
  // 立即加载一次
  loadStitchedImage()
  // 设置新的定时器
  intervalTimer = window.setInterval(
    () => {
      loadStitchedImage()
    },
    settings.value.interval * 60 * 1000
  )
}

// 添加防抖函数
function debounce<T extends (...args: any[]) => void>(func: T, delay: number): T {
  console.info('[debounce] 进入函数')
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

// 刷新按钮点击事件
function onRefresh() {
  console.info('[onRefresh] 进入函数')
  if (!isLoading.value) {
    loadStitchedImage()
  }
}

async function openSettings() {
  console.info('[openSettings] 进入函数')
  const store = await getStore()
  settings.value.interval = (await store.get('interval')) ?? 10
  settings.value.owmKey = (await store.get('owmKey')) ?? ''
  settings.value.owmUrl = (await store.get('owmUrl')) ?? ''
  showSettings.value = true
}

let handleResize: (() => void) | null = null

onMounted(async () => {
  console.log('onMounted')
  await attachConsole()
  console.info('[onMounted] 进入函数')
  startUpdates()
  resetIntervalTimer()
  handleResize = debounce(() => {
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
})

onUnmounted(() => {
  if (handleResize) {
    window.removeEventListener('resize', handleResize)
    console.log('[onUnmounted] 已移除窗口大小调整事件监听')
  }
})

// startUpdates 只更新时间，不再负责定时拉图
const startUpdates = () => {
  console.info('[startUpdates] 进入函数')
  // 每分钟更新时间
  setInterval(() => {
    currentTime.value = new Date()
  }, 60000)
}

async function onSaveImage() {
  console.info('[onSaveImage] 进入函数')
  // await ensureStoragePermission()
  if (!canvasRef.value) return
  // 获取 canvas 的 PNG 数据
  const dataUrl = canvasRef.value.toDataURL('image/png')
  // 正确的正则写法
  const base64 = dataUrl.replace(/^data:image\/png;base64,/, '')
  const bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0))

  const filePath = await save({
    title: '保存卫星图片',
    defaultPath: `satellite_${Date.now()}.png`,
    filters: [{ name: 'PNG 图片', extensions: ['png'] }],
  })
  if (!filePath) return

  try {
    await writeFile(filePath, bytes)
  } catch (e) {
    // 用 Tauri dialog 弹窗提示
    await save({
      title: '图片保存失败，请检查存储权限或空间',
    })
  }
}

let touchTimer: number | null = null

function onTouchStart() {
  console.info('[onTouchStart] 进入函数')
  touchTimer = window.setTimeout(() => {
    onSaveImage()
  }, 600)
}
function onTouchEnd() {
  console.info('[onTouchEnd] 进入函数')
  if (touchTimer) {
    clearTimeout(touchTimer)
    touchTimer = null
  }
}

// // 检查并请求存储权限
// async function ensureStoragePermission() {
//   // Android 13+ 推荐用 READ_MEDIA_IMAGES/VIDEO/AUDIO
//   const perms = [
//     Permission.ReadExternalStorage,
//     Permission.WriteExternalStorage
//   ]
//   for (const perm of perms) {
//     const status = await check(perm)
//     if (status !== 'granted') {
//       await request(perm)
//     }
//   }
// }
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
