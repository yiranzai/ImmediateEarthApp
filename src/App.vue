<script setup lang="ts">
import SatelliteImageViewer from './components/SatelliteImageViewer.vue'
import { useStore } from './store'
const store = useStore()
store.initApp()
</script>

<template>
  <div class="h-screen w-screen overflow-hidden">
    <SatelliteImageViewer />
  </div>
</template>
