<template>
  <div class="main-container">
    <!-- <h2>服务器管理</h2> -->
    <el-row :gutter="10">
      <el-col :span="6">
        <el-tooltip placement="top">
          <template #content>
            以下情况下需要输入设置的 `ATSH_KEY` 密钥<br>
            > 新增服务：强制加密服务器密码时使用<br>
            > 查看密码：解密时使用<br>
            其他时候不需要该密钥信息
          </template>
          <el-input v-model="atshKey" :disabled="atshKeyIsLocked" :type="atshKeyIsLocked ? 'password' : 'text'"
            :placeholder="atshKeyIsLocked ? '内容已锁定' : '请输入内容'" clearable>
            <template #append>
              <el-button :icon="atshKeyIsLocked ? 'Lock' : 'Unlock'" @click="atshKeyToggleLock"
                :type="atshKeyIsLocked ? 'primary' : ''" />
            </template>
          </el-input>
        </el-tooltip>
      </el-col>
      <el-col :span="2"><el-button type="primary" @click="openAddForm">新增服务器</el-button></el-col>
    </el-row>

    <!-- 服务器列表 -->
    <el-table :data="pagedServers" style="width: 100%" border size="small">
      <el-table-column prop="index" label="序号" width="60" />
      <el-table-column prop="name" label="名称" />
      <el-table-column prop="user" label="用户" />
      <el-table-column prop="ip" label="地址" />
      <el-table-column prop="port" label="端口" width="80" />
      <el-table-column label="密码" show-password>
        <template #default="scope">
          <el-input v-model="scope.row.password" type="password" show-password readonly class="no-border-input" />
        </template>
      </el-table-column>
      <el-table-column label="授权" width="60">
        <template #default="scope">
          <el-tag v-if="scope.row.authorized" type="success">是</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="note" label="备注" />
      <el-table-column label="操作" fixed="right" width="100">
        <template #default="scope">
          <!-- <el-button size="small" type="primary" @click="loginServer(scope.row)">登录</el-button> -->
          <!-- <el-button size="small" type="danger" :text="true" :icon="Delete" @click="deleteServer(scope.row.index)"> 删除 -->
          <el-space :size="6">
            <el-button size="small" type="primary" :icon="Connection" @click="loginServer(scope.row)"></el-button>
            <el-button size="small" type="danger" :icon="Delete" @click="deleteServer(scope.row.index)"></el-button>
          </el-space>
        </template>
      </el-table-column>
    </el-table>

    <!-- 分页 -->
    <div style="margin-top: 20px; text-align: right">
      <el-pagination @current-change="handleCurrentChange" :current-page="currentPage" :page-size="pageSize"
        :page-sizes="[10, 20, 50, 100]" @size-change="handleSizeChange" layout="prev, pager, next, sizes, total"
        :total="total" />
    </div>

    <!-- 新增服务器 -->
    <el-dialog v-model="showAddForm" title="" width="500px">
      <el-form :model="remoteForm" label-width="80px">
        <el-form-item label="名称">
          <el-tooltip placement="top" content="服务器名称，如 linux-amd64-develop">
            <el-input v-model="remoteForm.name" />
          </el-tooltip>
        </el-form-item>
        <el-form-item label="用户" required>
          <el-input v-model="remoteForm.user" />
        </el-form-item>
        <el-form-item label="地址" required>
          <el-input v-model="remoteForm.ip" />
        </el-form-item>
        <el-form-item label="端口" required>
          <el-input v-model.number="remoteForm.port" type="number" />
        </el-form-item>
        <el-form-item label="密码" required>
          <el-input v-model="remoteForm.password" type="password" show-password />
        </el-form-item>
        <el-form-item label="备注">
          <el-tooltip placement="top" content="其他信息，如 expired at 2025-11-11">
            <el-input v-model="remoteForm.note" type="textarea" />
          </el-tooltip>
        </el-form-item>
      </el-form>
      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showAddForm = false">取消</el-button>
          <el-button type="primary" @click="submitAddForm">确定</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Delete, Connection } from '@element-plus/icons-vue'

// =======================
// 🔹 类型定义
// =======================

interface Server {
  index: number
  user: string
  ip: string
  port: number
  password: string
  authorized: boolean
  name: string | null
  note: string | null
}

// =======================
// 🔹 响应式数据
// =======================


const atshKey = ref('')
const atshKeyIsLocked = ref(false)

const atshKeyToggleLock = () => {
  atshKeyIsLocked.value = !atshKeyIsLocked.value
  setAtshKey(atshKeyIsLocked.value, atshKey.value)
}


const currentPage = ref(1)
const pageSize = ref(5)
const total = ref(0)

const allServers = ref<Server[]>([])

const showAddForm = ref(false)
const remoteForm = ref<Server>({
  index: 0,
  user: '',
  ip: '',
  port: 22,
  password: '',
  authorized: false,
  name: null,
  note: null,
})


// =======================
// 🔹 计算属性
// =======================

const pagedServers = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  // console.log('start: ' + start);
  let slice = allServers.value.slice(start, start + pageSize.value)
  // console.log('slice: ' + slice);
  return slice
})

const handleCurrentChange = (val: number) => {
  currentPage.value = val
}

const handleSizeChange = (val: number) => {
  pageSize.value = val
  currentPage.value = 1
}

// =======================
// 🔹 方法实现
// =======================

// 添加 ATSH_KEY 环境变量
async function setAtshKey(isLocked: Boolean, key: string) {
  // console.log('isLocked: ' + isLocked)
  // console.log('key: ' + key)
  try {
    if (isLocked) {
      // 加入环境变量
      await invoke('set_atshkey', { key: key })
    } else {
      // 删除环境变量
      await invoke('set_atshkey', { key: null })
    }
    loadServers()
  } catch (err) {
    ElMessage.error('设置失败: ' + (err as Error).message)
  }
}


// 加载服务器列表
async function loadServers() {
  // console.log('加载服务器列表...')
  try {
    const data = await invoke<Server[]>('list_servers')
    // console.log('已加载 ' + JSON.stringify(data));
    ElMessage.info('已加载 ' + data.length + ' 条数据')
    allServers.value = data
    total.value = data.length
  } catch (err) {
    // console.error(err)
    ElMessage.error('加载失败: ' + (err as Error).message)
  }
}


// 打开新增表单
function openAddForm() {
  // const maxIndex = allServers.value.reduce((max, s) => (s.index > max ? s.index : max), 0)
  remoteForm.value = {
    index: 0,
    user: '',
    ip: '',
    port: 22,
    password: '',
    authorized: false,
    name: null,
    note: null,
  }
  showAddForm.value = true
}

// 提交新增
async function submitAddForm() {
  try {
    // ElMessage.info(JSON.stringify(remote.value))
    await invoke('add_server', { server: remoteForm.value })
    ElMessage.success('添加成功')
    showAddForm.value = false
    loadServers() // 刷新
  } catch (err) {
    ElMessage.error('添加失败: ' + (err as Error).message)
  }
}

// 删除服务器
async function deleteServer(index: number) {
  try {
    await ElMessageBox.confirm(`确定删除序号为 ${index} 的服务器？`, '警告', {
      type: 'warning',
      distinguishCancelAndClose: true,
      cancelButtonText: '取消',
      confirmButtonText: '确认'
    })
    await invoke('delete_server', { index: index })
    ElMessage.success(`序号 ${index} 的服务器删除成功`)
    loadServers()
  } catch (err) {
    if (err === 'cancel') {
      // 用户点击了取消按钮
      ElMessage.info('已取消删除操作')
    } else {
      // 其他错误（删除失败）
      ElMessage.error(`序号 ${index} 的服务器删除失败: ${err}`)
    }
  }
}

// 登录服务器
async function loginServer(server: Server) {
  try {
    await invoke('login_to_server', {
      index: server.index
    })
    ElMessage.success(`正在登录 ${server.user}@${server.ip}:${server.port}`)
  } catch (err) {
    ElMessage.error('登录失败: ' + (err as Error).message)
  }
}

// 初始化加载
loadServers()
</script>

<style scoped>
.main-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 20px;
}

.el-row {
  margin-bottom: 20px;
}

.el-row:last-child {
  margin-bottom: 0;
}

.el-col {
  border-radius: 4px;
}

.grid-content {
  border-radius: 4px;
  min-height: 36px;
}


.dialog-footer {
  text-align: right;
}

/* 隐藏边框和背景 */
.no-border-input .el-input__wrapper {
  box-shadow: none !important;
  background: transparent !important;
  padding: 0 !important;
}

/* 隐藏悬停/聚焦时的边框 */
.no-border-input .el-input__wrapper:hover,
.no-border-input .el-input__wrapper.is-focus {
  box-shadow: none !important;
}

.lockable-input {
  width: 200px;
}
</style>