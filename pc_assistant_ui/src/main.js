import { invoke } from '@tauri-apps/api'

// 桌宠容器元素
const petContainer = document.getElementById('petContainer')
const petMessage = document.getElementById('petMessage')

// 拖拽功能
let isDragging = false
let offsetX, offsetY

petContainer.addEventListener('mousedown', (e) => {
  isDragging = true
  offsetX = e.clientX - petContainer.getBoundingClientRect().left
  offsetY = e.clientY - petContainer.getBoundingClientRect().top
  petContainer.style.cursor = 'grabbing'
})

window.addEventListener('mousemove', (e) => {
  if (isDragging) {
    const x = e.clientX - offsetX
    const y = e.clientY - offsetY
    
    // 限制在屏幕范围内
    const maxX = window.innerWidth - petContainer.offsetWidth
    const maxY = window.innerHeight - petContainer.offsetHeight
    
    const clampedX = Math.max(0, Math.min(x, maxX))
    const clampedY = Math.max(0, Math.min(y, maxY))
    
    petContainer.style.left = `${clampedX}px`
    petContainer.style.top = `${clampedY}px`
    petContainer.style.bottom = 'auto'
    petContainer.style.right = 'auto'
  }
})

window.addEventListener('mouseup', () => {
  isDragging = false
  petContainer.style.cursor = 'move'
})

// 显示消息
function showMessage(message, duration = 3000) {
  petMessage.textContent = message
  petMessage.classList.add('show')
  
  setTimeout(() => {
    petMessage.classList.remove('show')
  }, duration)
}

// 点击桌宠触发交互
petContainer.addEventListener('click', async () => {
  showMessage('你好！有什么可以帮助你的吗？')
  // 这里可以实现语音识别或其他交互逻辑
  setTimeout(() => {
    showMessage('我是你的PC个人助手，随时为你服务！')
  }, 2000)
})

// 初始化
async function init() {
  showMessage('你好！我是您的PC个人助手')
  
  // 连接后端服务
  try {
    // 这里可以添加与后端服务的连接逻辑
    console.log('Connected to backend service')
  } catch (error) {
    console.error('Failed to connect to backend service:', error)
    showMessage('无法连接到后端服务')
  }
}

// 启动应用
init()
