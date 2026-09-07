<script setup lang="ts">
import { ref } from 'vue';

const emit = defineEmits<{ (e: 'browse', url: string): void }>();
const address = ref('192.168.1.1');
const port = ref(4840);

function browse() {
  const url = `opc.tcp://${address.value}:${port.value}`;
  emit('browse', url);
}

</script>

<template>
  <div class="address-bar">
    <span>opc.tcp://</span>
    <input type="text" placeholder="Search..." v-model="address" />
    <span>:</span>
    <input type="number" v-model="port" />
    <button @click="browse">Connect</button>
  </div>
</template>
