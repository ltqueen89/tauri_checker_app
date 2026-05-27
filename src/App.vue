<template>
  <div
    class="flex flex-col items-center min-h-screen bg-slate-900 text-white p-8"
  >
    <h1 class="text-3xl font-bold mb-8">Мониторинг сайтов</h1>

    <div class="w-full max-w-2xl space-y-2">
      <div
        class="flex justify-between items-center px-4 py-2 text-slate-400 text-xs uppercase font-bold"
      >
        <span class="w-1/3">Сайт</span>
        <span class="w-1/4 text-center">HTTP</span>
        <span class="w-1/4 text-center">TCP (443)</span>
        <span class="w-10"></span>
      </div>

      <div
        v-for="(site, index) in sites"
        :key="index"
        class="flex justify-between items-center p-4 hover:bg-slate-700 bg-slate-800 rounded-xl border border-slate-700 transition-colors"
      >
        <div class="w-1/3 truncate">
          <span class="font-medium text-slate-200">{{ site.url }}</span>
        </div>

        <div
          class="w-1/4 text-center font-mono text-sm"
          :class="site.colorHttp"
        >
          {{ site.http }}
        </div>

        <div class="w-1/4 text-center font-mono text-sm" :class="site.colorTcp">
          {{ site.tcp }}
        </div>

        <button
          @click="pingSite(index)"
          class="ml-4 text-xs bg-slate-700 hover:bg-slate-600 p-2 rounded-lg transition-transform active:scale-95"
        >
          🔄
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Функция для создания объекта сайта, чтобы не писать 30 раз одно и то же
const createSite = (url: string) => ({
  url,
  http: "Ожидание",
  tcp: "Ожидание",
  colorHttp: "text-gray-500",
  colorTcp: "text-gray-500",
});

const sites = ref([
  createSite("google.com"),
  createSite("vk.com"),
  createSite("discord.com"),
  createSite("yandex.ru"),
  createSite("youtube.com"),
  createSite("ozon.ru"),
  createSite("wildberries.ru"),
  createSite("avito.ru"),
  createSite("ok.ru"),
  createSite("mail.ru"),
  createSite("sberbank.ru"),
  createSite("tinkoff.ru"),
  createSite("gosuslugi.ru"),
  createSite("telegram.org"),
  createSite("dzen.ru"),
  createSite("kinopoisk.ru"),
  createSite("github.com"),
  createSite("habr.com"),
  createSite("pikabu.ru"),
  createSite("rt.com"),
  createSite("rbc.ru"),
  createSite("lenta.ru"),
  createSite("dns-shop.ru"),
  createSite("mvideo.ru"),
  createSite("vprok.ru"),
  createSite("citilink.ru"),
  createSite("hh.ru"),
  createSite("aviasales.ru"),
  createSite("cian.ru"),
  createSite("rutracker.org"),
  createSite("steampowered.com"),
]);

async function pingSite(index: number) {
  const site = sites.value[index];
  site.http = "⏳";
  site.tcp = "⏳";
  site.colorHttp = "text-yellow-400";
  site.colorTcp = "text-yellow-400";

  try {
    const res = (await invoke("check_site_status", { url: site.url })) as {
      http: string;
      tcp: string;
    };

    site.http = res.http;
    site.tcp = res.tcp;

    // Обновляем цвета на основе ответа из Rust
    site.colorHttp = res.http.includes("200")
      ? "text-green-400"
      : "text-red-400";
    site.colorTcp = res.tcp.includes("OK") ? "text-green-400" : "text-red-400";
  } catch (err) {
    site.http = "Error";
    site.tcp = "Error";
    site.colorHttp = "text-red-600";
    site.colorTcp = "text-red-600";
  }
}

onMounted(async () => {
  // Проверяем по очереди, чтобы не перегружать мост Tauri
  for (let i = 0; i < sites.value.length; i++) {
    pingSite(i);
    // Небольшая задержка перед следующим сайтом для красоты анимации
    await new Promise((r) => setTimeout(r, 100));
  }
});
</script>
