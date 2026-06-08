<template>
  <div class="min-h-screen bg-slate-900 text-white p-8 font-sans">
    <div class="max-w-4xl mx-auto space-y-6">
      <div class="flex justify-between items-center">
        <h1
          class="text-3xl font-extrabold tracking-tight indigo-400 bg-clip-text text-transparent"
        >
          Панель Мониторинга
        </h1>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div
          class="bg-slate-800/60 border border-slate-700/50 p-4 rounded-xl shadow-sm flex items-center space-x-4"
        >
          <div class="p-3 bg-blue-500/10 text-blue-400 rounded-lg text-xl">
            📊
          </div>
          <div>
            <div class="text-2xl font-bold font-mono">{{ stats.total }}</div>
            <div class="text-xs text-slate-400 font-medium">Всего сайтов</div>
          </div>
        </div>
        <div
          class="bg-slate-800/60 border border-slate-700/50 p-4 rounded-xl shadow-sm flex items-center space-x-4"
        >
          <div
            class="p-3 bg-emerald-500/10 text-emerald-400 rounded-lg text-xl"
          >
            ✅
          </div>
          <div>
            <div class="text-2xl font-bold font-mono text-emerald-400">
              {{ stats.online }}
            </div>
            <div class="text-xs text-slate-400 font-medium">
              Доступны (200 OK)
            </div>
          </div>
        </div>
        <div
          class="bg-slate-800/60 border border-slate-700/50 p-4 rounded-xl shadow-sm flex items-center space-x-4"
        >
          <div class="p-3 bg-rose-500/10 text-rose-400 rounded-lg text-xl">
            🚨
          </div>
          <div>
            <div class="text-2xl font-bold font-mono text-rose-400">
              {{ stats.offline }}
            </div>
            <div class="text-xs text-slate-400 font-medium">С ошибками</div>
          </div>
        </div>
      </div>

      <div
        class="bg-slate-800 p-4 rounded-2xl border border-slate-700/60 shadow-xl space-y-4"
      >
        <div class="relative">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Поиск или ввод нового домена..."
            class="w-full pl-5 pr-4 py-2.5 bg-slate-900 border border-slate-700 rounded-xl text-slate-200 placeholder-slate-500 focus:outline-none focus:border-slate-500 transition-colors"
          />
        </div>

        <div
          class="flex flex-col sm:flex-row justify-between gap-3 pt-1 border-t border-slate-700/40"
        >
          <div
            class="flex bg-slate-900 p-1 rounded-xl border border-slate-700/50 self-start"
          >
            <button
              @click="currentFilter = 'all'"
              :class="
                currentFilter === 'all'
                  ? 'bg-slate-700 text-white font-medium'
                  : 'text-slate-400 hover:text-slate-200'
              "
              class="px-3 py-1.5 text-xs rounded-lg transition-all"
            >
              Все
            </button>
            <button
              @click="currentFilter = 'errors'"
              :class="
                currentFilter === 'errors'
                  ? 'bg-rose-600/30 text-rose-400 border border-rose-500/30 font-medium'
                  : 'text-slate-400 hover:text-slate-200'
              "
              class="px-3 py-1.5 text-xs rounded-lg transition-all mx-1"
            >
              Проблемные ({{ stats.offline }})
            </button>
            <button
              @click="currentFilter = 'pending'"
              :class="
                currentFilter === 'pending'
                  ? 'bg-slate-700 text-white font-medium'
                  : 'text-slate-400 hover:text-slate-200'
              "
              class="px-3 py-1.5 text-xs rounded-lg transition-all"
            >
              Ожидание
            </button>
          </div>

          <div class="flex items-center space-x-2 self-end sm:self-auto">
            <span class="text-xs text-slate-400">Сортировка:</span>
            <select
              v-model="currentSort"
              class="bg-slate-900 border border-slate-700 rounded-lg px-2.5 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-slate-500 cursor-pointer"
            >
              <option value="alphabet">По алфавиту</option>
              <option value="ping_asc">По пингу (быстрые)</option>
              <option value="ping_desc">По пингу (медленные)</option>
            </select>
          </div>
        </div>
      </div>

      <div
        v-if="isNewSiteQuery"
        class="p-4 bg-indigo-600/15 border border-indigo-500/30 rounded-xl flex justify-between items-center transition-all"
      >
        <div class="flex flex-col">
          <span
            class="text-[10px] text-indigo-400 font-bold uppercase tracking-wider"
            >Новый хост</span
          >
          <span class="text-slate-200 font-medium truncate max-w-md">{{
            searchQuery
          }}</span>
        </div>
        <button
          @click="addAndPingNewSite"
          class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-xs font-semibold rounded-lg transition-transform active:scale-95 shadow-md shadow-indigo-600/10"
        >
          Проверить и добавить
        </button>
      </div>

      <div class="space-y-2">
        <div
          class="flex justify-between items-center px-4 py-2 text-slate-500 text-xs uppercase font-bold tracking-wider"
        >
          <span class="w-1/4">Сайт</span>
          <span class="w-1/6 text-center">HTTP Status</span>
          <span class="w-1/6 text-center">TCP (443)</span>
          <span
            class="w-1/4 text-center flex items-center justify-center gap-2"
          >
            <span class="flex items-center gap-1">
              <span class="w-2 h-0.5 bg-emerald-400 inline-block"></span>HTTP
            </span>
            <span class="flex items-center gap-1">
              <span class="w-2 h-0.5 bg-sky-400 inline-block"></span>TCP
            </span>
          </span>
          <span class="w-21.5 text-center">Действия</span>
        </div>

        <div
          v-for="site in finalSites"
          :key="site.id"
          class="flex justify-between items-center p-4 bg-slate-800/40 hover:bg-slate-800/80 rounded-xl border border-slate-800 hover:border-slate-700/80 transition-all shadow-sm"
        >
          <div class="w-1/4 truncate">
            <span
              class="font-medium text-slate-200 hover:text-white cursor-pointer"
              >{{ site.url }}</span
            >
          </div>

          <div
            class="w-1/6 text-center font-mono text-xs font-bold"
            :class="site.colorHttp"
          >
            {{ site.http }}
          </div>

          <div
            class="w-1/6 text-center font-mono text-xs font-bold"
            :class="site.colorTcp"
          >
            {{ site.tcp }}
          </div>

          <div class="w-1/4 flex justify-center h-10 items-center">
            <Sparkline
              :values="site.telemetry"
              :tcpValues="site.tcpTelemetry"
              :width="140"
              :height="36"
            />
          </div>

          <div class="flex items-center gap-1.5">
            <button
              @click="pingSite(site)"
              title="Проверить сейчас"
              class="w-13 h-9 flex items-center justify-center text-xs bg-slate-800 hover:bg-slate-700 border border-slate-700/40 rounded-lg transition-transform active:scale-95"
            >
              Обновить
            </button>
            <button
              @click="deleteSite(site)"
              title="Удалить сайт"
              class="w-13 h-9 flex items-center justify-center text-xs bg-slate-800 hover:bg-rose-600/30 hover:text-rose-400 hover:border-rose-500/30 border border-slate-700/40 rounded-lg transition-transform active:scale-95"
            >
              Удалить
            </button>
          </div>
        </div>

        <div
          v-if="finalSites.length === 0 && !isNewSiteQuery"
          class="text-center py-16 text-slate-500 bg-slate-800/20 rounded-xl border border-slate-800/50 dashed"
        >
          Ничего не найдено по заданным фильтрам
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Sparkline from "./Sparkline.vue";

interface MonitorSite {
  id: number;
  url: string;
  http: string;
  tcp: string;
  colorHttp: string;
  colorTcp: string;
  telemetry: number[];
  tcpTelemetry: number[];
}

interface TelemetryPoint {
  http_time: number;
  tcp_time: number;
  tcp_ok: boolean;
}

interface DbSite {
  id: number;
  url: string;
}

const createSite = (id: number, url: string): MonitorSite => ({
  id,
  url,
  http: "Ожидание",
  tcp: "Ожидание",
  colorHttp: "text-slate-500",
  colorTcp: "text-slate-500",
  telemetry: [],
  tcpTelemetry: [],
});

const sites = ref<MonitorSite[]>([]);

const searchQuery = ref("");
const currentFilter = ref<"all" | "errors" | "pending">("all");
const currentSort = ref<"alphabet" | "ping_asc" | "ping_desc">("alphabet");

const isPending = (s: MonitorSite) =>
  s.http === "Ожидание" || s.http === "Загрузка";
const isOnline = (s: MonitorSite) => {
  if (isPending(s)) return false;
  const code = parseInt(s.http, 10);
  return code >= 200 && code < 400;
};

const stats = computed(() => {
  const total = sites.value.length;
  const pending = sites.value.filter(isPending).length;
  const online = sites.value.filter(isOnline).length;
  const offline = total - online - pending;
  return { total, online, offline, pending };
});

const isNewSiteQuery = computed(() => {
  const query = searchQuery.value.toLowerCase().trim();
  if (query.length < 4) return false;
  return !sites.value.some((s) => s.url.toLowerCase() === query);
});

const finalSites = computed(() => {
  let result = [...sites.value];
  const query = searchQuery.value.toLowerCase().trim();

  if (query) {
    result = result.filter((s) => s.url.toLowerCase().includes(query));
  }

  if (currentFilter.value === "errors") {
    result = result.filter((s) => !isOnline(s) && !isPending(s));
  } else if (currentFilter.value === "pending") {
    result = result.filter(isPending);
  }

  result.sort((a, b) => {
    if (currentSort.value === "alphabet") {
      return a.url.localeCompare(b.url);
    }
    const pingA = a.telemetry[a.telemetry.length - 1] || 0;
    const pingB = b.telemetry[b.telemetry.length - 1] || 0;

    if (currentSort.value === "ping_asc") {
      if (pingA === 0) return 1;
      if (pingB === 0) return -1;
      return pingA - pingB;
    } else {
      return pingB - pingA;
    }
  });

  return result;
});

async function addAndPingNewSite() {
  const cleanUrl = searchQuery.value.trim().toLowerCase();
  if (!cleanUrl) return;

  try {
    const generatedId = await invoke<number>("add_site_to_db", {
      url: cleanUrl,
    });

    const newSite = createSite(generatedId, cleanUrl);

    sites.value.unshift(newSite);
    searchQuery.value = "";
    currentFilter.value = "all";

    await pingSite(newSite);
  } catch (err) {
    console.error("Не удалось сохранить сайт в базу данных:", err);
    alert(`Ошибка БД: ${err}`);
  }
}

async function deleteSite(site: MonitorSite) {
  if (
    !confirm(
      `Удалить «${site.url}» и всю историю проверок? Это действие необратимо.`,
    )
  ) {
    return;
  }

  try {
    await invoke("delete_site", { id: site.id });
    sites.value = sites.value.filter((s) => s.id !== site.id);
  } catch (err) {
    console.error(`Не удалось удалить ${site.url}:`, err);
    alert(`Ошибка удаления: ${err}`);
  }
}

async function loadSiteHistory(site: MonitorSite) {
  try {
    const history = await invoke<TelemetryPoint[]>("get_site_history", {
      siteId: site.id,
    });
    site.telemetry = history.map((point) => point.http_time);
    site.tcpTelemetry = history.map((point) => point.tcp_time);
  } catch (err) {
    console.error(`Ошибка истории для ${site.url}:`, err);
  }
}

async function pingSite(site: MonitorSite) {
  site.http = "Загрузка";
  site.tcp = "Загрузка";
  site.colorHttp = "text-yellow-400";
  site.colorTcp = "text-yellow-400";

  try {
    const res = await invoke<{ http: string; tcp: string }>(
      "check_site_status",
      { url: site.url },
    );
    site.http = res.http;
    site.tcp = res.tcp;
    site.colorHttp = isOnline(site) ? "text-emerald-400" : "text-rose-400";
    site.colorTcp = res.tcp.includes("OK")
      ? "text-emerald-400"
      : "text-rose-400";
    await loadSiteHistory(site);
  } catch (err) {
    site.http = "Error";
    site.tcp = "Error";
    site.colorHttp = "text-rose-600";
    site.colorTcp = "text-rose-600";
  }
}

onMounted(async () => {
  try {
    const dbSites = await invoke<DbSite[]>("get_all_sites");

    sites.value = dbSites.map((s) => createSite(s.id, s.url));

    await Promise.all(sites.value.map((site) => loadSiteHistory(site)));

    for (const site of sites.value) {
      pingSite(site);
      await new Promise((r) => setTimeout(r, 80));
    }
  } catch (err) {
    console.error(
      "Критическая ошибка при загрузке сайтов из базы данных:",
      err,
    );
  }

  setInterval(() => {
    sites.value.forEach((site) => loadSiteHistory(site));
  }, 60000);
});
</script>
