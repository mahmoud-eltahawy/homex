// app.tsx – Fully responsive, polished, bug‑free
import { Router, Route, A, useParams, useNavigate, useSearchParams } from '@solidjs/router';
import { createResource, createSignal, createEffect, Suspense, For, Show, onMount, onCleanup } from 'solid-js';

// ---------- Types ----------
interface Episode {
  id: number;
  season: number;
  episode: number;
  title: string;
  filePath: string;
}

interface Media {
  id: number;
  title: string;
  type: 'movie' | 'series';
  poster: string;
  filePath: string;
  size: string;
  description?: string;
  year?: number;
  duration?: string;
  episodes?: Episode[];
}

// ---------- Mock Data with reliable video URL ----------
const TEST_VIDEO = 'https://www.w3schools.com/html/mov_bbb.mp4';

const mockMovies: Media[] = [
  {
    id: 1,
    title: 'Inception',
    type: 'movie',
    poster: 'https://picsum.photos/seed/inception/300/450',
    filePath: TEST_VIDEO,
    size: '2.1 جيجابايت',
    description: 'لص يسرق أسرار الشركات من خلال تقنية مشاركة الأحلام.',
    year: 2010,
    duration: 'ساعتان و28 دقيقة'
  },
  {
    id: 2,
    title: 'The Matrix',
    type: 'movie',
    poster: 'https://picsum.photos/seed/matrix/300/450',
    filePath: TEST_VIDEO,
    size: '1.8 جيجابايت',
    description: 'هاكر كمبيوتر يكتشف حقيقة الواقع.',
    year: 1999,
    duration: 'ساعتان و16 دقيقة'
  },
  {
    id: 3,
    title: 'Interstellar',
    type: 'movie',
    poster: 'https://picsum.photos/seed/interstellar/300/450',
    filePath: TEST_VIDEO,
    size: '3.1 جيجابايت',
    description: 'فريق من المستكشفين يسافرون عبر ثقب دودي في الفضاء.',
    year: 2014,
    duration: 'ساعتان و49 دقيقة'
  },
  {
    id: 4,
    title: 'The Dark Knight',
    type: 'movie',
    poster: 'https://picsum.photos/seed/darkknight/300/450',
    filePath: TEST_VIDEO,
    size: '2.5 جيجابايت',
    description: 'عندما يهدد الجوكر مدينة غوثام بالدمار.',
    year: 2008,
    duration: 'ساعتان و32 دقيقة'
  },
  {
    id: 5,
    title: 'Pulp Fiction',
    type: 'movie',
    poster: 'https://picsum.photos/seed/pulpfiction/300/450',
    filePath: TEST_VIDEO,
    size: '1.9 جيجابايت',
    description: 'تتشابك حياة اثنين من القتلة وملاكم وزوجين من اللصوص.',
    year: 1994,
    duration: 'ساعتان و34 دقيقة'
  },
];

const mockSeries: Media[] = [
  {
    id: 101,
    title: 'Breaking Bad',
    type: 'series',
    poster: 'https://picsum.photos/seed/breakingbad/300/450',
    filePath: '/media/series/breakingbad/',
    size: '45 جيجابايت (5 مواسم)',
    description: 'مدرس كيمياء يتحول إلى تاجر مخدرات.',
    year: 2008,
    duration: '5 مواسم',
    episodes: [
      { id: 1011, season: 1, episode: 1, title: 'Pilot', filePath: TEST_VIDEO },
      { id: 1012, season: 1, episode: 2, title: 'Cat\'s in the Bag...', filePath: TEST_VIDEO },
      { id: 1013, season: 1, episode: 3, title: '...And the Bag\'s in the River', filePath: TEST_VIDEO },
      { id: 1014, season: 2, episode: 1, title: 'Seven Thirty-Seven', filePath: TEST_VIDEO },
      { id: 1015, season: 2, episode: 2, title: 'Grilled', filePath: TEST_VIDEO },
    ]
  },
  {
    id: 102,
    title: 'Stranger Things',
    type: 'series',
    poster: 'https://picsum.photos/seed/strangerthings/300/450',
    filePath: '/media/series/strangerthings/',
    size: '32 جيجابايت (4 مواسم)',
    description: 'مجموعة من الأطفال يكشفون أسرارًا خارقة في بلدتهم.',
    year: 2016,
    duration: '4 مواسم',
    episodes: [
      { id: 1021, season: 1, episode: 1, title: 'Chapter One: Will Byers', filePath: TEST_VIDEO },
      { id: 1022, season: 1, episode: 2, title: 'Chapter Two: The Weirdo on Maple Street', filePath: TEST_VIDEO },
    ]
  },
  {
    id: 103,
    title: 'The Crown',
    type: 'series',
    poster: 'https://picsum.photos/seed/thecrown/300/450',
    filePath: '/media/series/thecrown/',
    size: '28 جيجابايت (4 مواسم)',
    description: 'عهد الملكة إليزابيث الثانية.',
    year: 2016,
    duration: '4 مواسم',
    episodes: [
      { id: 1031, season: 1, episode: 1, title: 'Wolferton Splash', filePath: TEST_VIDEO },
    ]
  },
  {
    id: 104,
    title: 'Game of Thrones',
    type: 'series',
    poster: 'https://picsum.photos/seed/got/300/450',
    filePath: '/media/series/got/',
    size: '68 جيجابايت (8 مواسم)',
    description: 'عائلات نبيلة تتصارع على السيطرة على ويستروس.',
    year: 2011,
    duration: '8 مواسم',
    episodes: [
      { id: 1041, season: 1, episode: 1, title: 'Winter Is Coming', filePath: TEST_VIDEO },
      { id: 1042, season: 1, episode: 2, title: 'The Kingsroad', filePath: TEST_VIDEO },
    ]
  },
];

// ---------- API Simulation ----------
const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));
const fetchMovies = async (): Promise<Media[]> => { await delay(300); return mockMovies; };
const fetchSeries = async (): Promise<Media[]> => { await delay(300); return mockSeries; };
const fetchAllMedia = async (): Promise<Media[]> => { await delay(300); return [...mockMovies, ...mockSeries]; };
const fetchMediaDetail = async (type: string, id: string): Promise<Media | undefined> => {
  await delay(200);
  const all = type === 'movie' ? mockMovies : mockSeries;
  return all.find(m => m.id === Number(id));
};

// ---------- SVG Icons ----------
const SearchIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/></svg>
);
const MovieIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"/></svg>
);
const SeriesIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/></svg>
);
const DownloadIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5 inline-block ms-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
);
const PlayIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5 inline-block ms-1" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
);
const PauseIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5 inline-block ms-1" fill="currentColor" viewBox="0 0 24 24"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
);
const ClockIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 md:h-4 md:w-4 inline-block ms-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
);
const UploadIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 md:h-6 md:w-6 inline-block ms-1" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/></svg>
);
const DeleteIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5 inline-block" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></svg>
);
const UpArrow = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 md:h-4 md:w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/></svg>
);
const DownArrow = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 md:h-4 md:w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
);
const SortIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4h13M3 8h9m-9 4h9m5-4v12m0 0l-4-4m4 4l4-4"/></svg>
);
const VolumeIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/></svg>
);
const MuteIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" clip-rule="evenodd"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/></svg>
);
const FullscreenIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5v-4m0 4h-4m4 0l-5-5"/></svg>
);
const FullscreenExitIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 md:h-5 md:w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 9V4M9 4H4M9 4l5 5M15 15V20M15 20h5M15 20l-5-5M9 15v5M9 15H4M9 15l5 5M15 9V4M15 4h5M15 4l-5 5"/></svg>
);

// ---------- Layout ----------
function Layout(props: any) {
  return (
    <div class="flex flex-col min-h-screen bg-gray-950" dir="rtl">
      <Navbar />
      <main class="flex-1 bg-gradient-to-b from-gray-950 via-gray-900 to-black pt-20 md:pt-24 lg:pt-28 pb-8 md:pb-12">
        {props.children}
      </main>
      <Footer />
    </div>
  );
}

// ---------- Navbar ----------
function Navbar() {
  const navigate = useNavigate();
  const [searchTerm, setSearchTerm] = createSignal('');
  const [searchOpen, setSearchOpen] = createSignal(false);

  const handleSearch = (e: Event) => {
    e.preventDefault();
    const term = searchTerm().trim();
    if (term) {
      navigate(`/search?q=${encodeURIComponent(term)}`);
      setSearchOpen(false);
    }
  };

  return (
    <nav class="fixed top-0 start-0 end-0 z-50 backdrop-blur-2xl bg-black/50 border-b border-white/10 shadow-lg shadow-black/20">
      <div class="max-w-7xl mx-auto px-3 sm:px-4 lg:px-8">
        <div class="flex items-center justify-between h-14 sm:h-16 md:h-20">
          <A href="/" class="flex items-center gap-1.5 text-xl sm:text-2xl md:text-3xl font-extrabold tracking-tight">
            <span class="bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">وسائطي</span>
          </A>
          <div class="hidden md:flex items-center gap-1">
            <A href="/movies" class="px-3 py-1.5 rounded-xl text-sm font-medium text-gray-300 hover:text-white hover:bg-white/10 transition-all duration-200">أفلام</A>
            <A href="/series" class="px-3 py-1.5 rounded-xl text-sm font-medium text-gray-300 hover:text-white hover:bg-white/10 transition-all duration-200">مسلسلات</A>
            <div class={`relative me-2 transition-all duration-300 ease-in-out ${searchOpen() ? 'w-48 lg:w-56' : 'w-8'}`}>
              <form onSubmit={handleSearch} class="flex items-center">
                <button type="button" onClick={() => setSearchOpen(!searchOpen())} class="absolute end-1 top-1/2 -translate-y-1/2 p-1 rounded-full text-gray-400 hover:text-white hover:bg-white/10 transition-colors"><SearchIcon /></button>
                <input type="text" value={searchTerm()} onInput={(e) => setSearchTerm(e.currentTarget.value)} onFocus={() => setSearchOpen(true)} onBlur={() => { if (!searchTerm()) setSearchOpen(false); }} placeholder="ابحث..." class={`w-full bg-white/5 backdrop-blur-md text-white placeholder-gray-500 rounded-full py-1.5 md:py-2 pe-8 md:pe-10 ps-3 md:ps-4 text-xs md:text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:bg-white/10 transition-all ${searchOpen() ? 'opacity-100' : 'opacity-0 pointer-events-none'}`} />
              </form>
            </div>
          </div>
          <div class="md:hidden flex items-center gap-1">
            <form onSubmit={handleSearch} class="relative flex items-center">
              <input type="text" value={searchTerm()} onInput={(e) => setSearchTerm(e.currentTarget.value)} placeholder="ابحث..." class="w-24 sm:w-32 bg-white/10 backdrop-blur-md text-white placeholder-gray-400 rounded-full py-1 pe-7 ps-2 text-xs focus:outline-none focus:ring-1 focus:ring-cyan-400" />
              <button type="submit" class="absolute end-1.5 top-1/2 -translate-y-1/2 text-gray-400"><SearchIcon /></button>
            </form>
          </div>
        </div>
        <div class="md:hidden flex gap-1 pb-1.5">
          <A href="/movies" class="flex-1 text-center py-1 rounded-lg text-xs font-medium text-gray-300 hover:text-white hover:bg-white/10 transition">أفلام</A>
          <A href="/series" class="flex-1 text-center py-1 rounded-lg text-xs font-medium text-gray-300 hover:text-white hover:bg-white/10 transition">مسلسلات</A>
        </div>
      </div>
    </nav>
  );
}

// ---------- Footer ----------
function Footer() {
  return (
    <footer class="bg-gray-950/80 backdrop-blur-md border-t border-white/10 mt-auto">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 md:py-12">
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-6 md:gap-8">
          <div class="space-y-2 md:space-y-3">
            <A href="/" class="text-xl md:text-2xl font-extrabold tracking-tight">
              <span class="bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">وسائطي</span>
            </A>
            <p class="text-gray-400 text-xs md:text-sm max-w-xs">
              خادم السينما الشخصي الخاص بك — شاهد، حمّل، واستمتع بمجموعتك في أي وقت.
            </p>
          </div>
          <div>
            <h3 class="text-white font-semibold text-sm md:text-base mb-2 md:mb-4">تصفح</h3>
            <ul class="space-y-1.5 text-xs md:text-sm">
              <li><A href="/movies" class="text-gray-400 hover:text-white transition">أفلام</A></li>
              <li><A href="/series" class="text-gray-400 hover:text-white transition">مسلسلات</A></li>
              <li><A href="/search" class="text-gray-400 hover:text-white transition">بحث</A></li>
            </ul>
          </div>
          <div>
            <h3 class="text-white font-semibold text-sm md:text-base mb-2 md:mb-4">المكتبة</h3>
            <ul class="space-y-1.5 text-xs md:text-sm">
              <li><A href="/upload" class="text-gray-400 hover:text-white transition">رفع وسائط</A></li>
              <li><A href="/settings" class="text-gray-400 hover:text-white transition">الإعدادات</A></li>
              <li><span class="text-gray-500 cursor-default">v1.0.0</span></li>
            </ul>
          </div>
        </div>
        <div class="mt-6 md:mt-10 pt-4 md:pt-6 border-t border-white/5 text-center text-gray-500 text-xs md:text-sm">
          <p>© 2025 وسائطي. صُنع بكل ❤️ لشبكتك المنزلية.</p>
        </div>
      </div>
    </footer>
  );
}

// ---------- Media Card ----------
function MediaCard({ item, type }: { item: Media; type: string }) {
  return (
    <A href={`/${type}/${item.id}`} class="group relative flex flex-col overflow-hidden rounded-xl sm:rounded-2xl bg-gray-900/60 backdrop-blur-sm border border-white/5 shadow-2xl hover:shadow-cyan-500/10 transition-all duration-500 hover:scale-[1.02] hover:-translate-y-1">
      <div class="aspect-[2/3] relative overflow-hidden">
        <img src={item.poster || 'https://via.placeholder.com/300x450?text=لا+صورة'} alt={item.title} class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-110" loading="lazy" onError={(e) => { (e.target as HTMLImageElement).src = 'https://via.placeholder.com/300x450?text=لا+صورة'; }} />
        <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-2 sm:p-4">
          <div class="transform translate-y-2 sm:translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
            <h3 class="text-white font-bold text-xs sm:text-base md:text-lg leading-tight line-clamp-2">{item.title}</h3>
            <div class="flex items-center gap-1 sm:gap-2 mt-0.5 sm:mt-1 text-gray-300 text-[10px] sm:text-sm">
              <Show when={item.year}><span>{item.year}</span></Show>
              <span class="flex items-center"><ClockIcon />{item.duration}</span>
            </div>
          </div>
        </div>
        <div class="absolute top-1.5 sm:top-3 end-1.5 sm:end-3 bg-black/60 backdrop-blur-md rounded-full px-1.5 py-0.5 sm:px-2 sm:py-0.5 text-[8px] sm:text-xs font-semibold text-white flex items-center gap-0.5 sm:gap-1">
          {item.type === 'movie' ? <MovieIcon /> : <SeriesIcon />}
          {item.type === 'movie' ? 'فيلم' : 'مسلسل'}
        </div>
      </div>
      <div class="p-2 sm:p-4 flex flex-col gap-0.5 sm:gap-1">
        <h3 class="text-gray-200 font-semibold truncate text-[10px] sm:text-sm">{item.title}</h3>
        <div class="flex items-center justify-between text-gray-500 text-[8px] sm:text-xs">
          <span class="flex items-center gap-0.5 sm:gap-1"><Show when={item.year}>{item.year} · </Show>{item.size}</span>
          <span class="text-cyan-400 text-[8px] sm:text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity">← التفاصيل</span>
        </div>
      </div>
    </A>
  );
}

// ---------- Skeleton Loader ----------
function CardSkeleton() {
  return (
    <div class="animate-pulse rounded-xl sm:rounded-2xl bg-gray-800/50 border border-white/5 overflow-hidden">
      <div class="aspect-[2/3] bg-gray-700/50" />
      <div class="p-2 sm:p-4 space-y-1.5 sm:space-y-2">
        <div class="h-2 sm:h-3 bg-gray-700 rounded w-3/4" />
        <div class="h-1.5 sm:h-2 bg-gray-700 rounded w-1/2" />
      </div>
    </div>
  );
}

// ---------- Video Player Component (LTR, with touch‑friendly controls) ----------
function VideoPlayer(props: { src: string; title?: string; onEnded?: () => void }) {
  let videoRef: HTMLVideoElement | undefined;
  let controlsTimeout: number | undefined;

  const [playing, setPlaying] = createSignal(false);
  const [currentTime, setCurrentTime] = createSignal(0);
  const [duration, setDuration] = createSignal(0);
  const [volume, setVolume] = createSignal(1);
  const [muted, setMuted] = createSignal(false);
  const [fullscreen, setFullscreen] = createSignal(false);
  const [controlsVisible, setControlsVisible] = createSignal(true);

  // Auto‑hide controls after 3 seconds of inactivity
  const startHideTimer = () => {
    if (controlsTimeout) clearTimeout(controlsTimeout);
    controlsTimeout = setTimeout(() => {
      setControlsVisible(false);
    }, 3000);
  };

  const showControls = () => {
    setControlsVisible(true);
    startHideTimer();
  };

  const toggleControls = () => {
    if (controlsVisible()) {
      setControlsVisible(false);
      if (controlsTimeout) clearTimeout(controlsTimeout);
    } else {
      showControls();
    }
  };

  // Reset timer on any user interaction
  const handleUserInteraction = () => {
    showControls();
  };

  // Cleanup timer on unmount
  onCleanup(() => {
    if (controlsTimeout) clearTimeout(controlsTimeout);
  });

  // Video event handlers
  const handleLoadedMetadata = () => {
    if (videoRef) {
      setDuration(videoRef.duration);
    }
  };

  const handleTimeUpdate = () => {
    if (videoRef) {
      setCurrentTime(videoRef.currentTime);
    }
  };

  const togglePlay = () => {
    if (!videoRef) return;
    if (playing()) {
      videoRef.pause();
    } else {
      videoRef.play();
    }
    setPlaying(!playing());
    handleUserInteraction();
  };

  const handleSeek = (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    const val = parseFloat(input.value);
    if (videoRef && !isNaN(val)) {
      videoRef.currentTime = val;
      setCurrentTime(val);
    }
    handleUserInteraction();
  };

  const handleVolumeChange = (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    const val = parseFloat(input.value);
    if (!isNaN(val)) {
      setVolume(val);
      if (videoRef) {
        videoRef.volume = val;
        videoRef.muted = val === 0;
        setMuted(val === 0);
      }
    }
    handleUserInteraction();
  };

  const toggleMute = () => {
    if (!videoRef) return;
    if (muted()) {
      videoRef.muted = false;
      setMuted(false);
      videoRef.volume = volume();
    } else {
      videoRef.muted = true;
      setMuted(true);
    }
    handleUserInteraction();
  };

  const toggleFullscreen = () => {
    if (!videoRef) return;
    if (!document.fullscreenElement) {
      videoRef.requestFullscreen?.();
      setFullscreen(true);
    } else {
      document.exitFullscreen?.();
      setFullscreen(false);
    }
    handleUserInteraction();
  };

  onMount(() => {
    const handleFullscreenChange = () => {
      setFullscreen(!!document.fullscreenElement);
    };
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    // Start with controls visible
    showControls();
    return () => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
      if (controlsTimeout) clearTimeout(controlsTimeout);
    };
  });

  const formatTime = (time: number) => {
    if (isNaN(time)) return '00:00';
    const mins = Math.floor(time / 60);
    const secs = Math.floor(time % 60);
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  };

  createEffect(() => {
    if (videoRef) {
      videoRef.load();
      setPlaying(false);
      setCurrentTime(0);
    }
  });

  return (
    <div dir="ltr" class="relative bg-black rounded-lg sm:rounded-xl overflow-hidden shadow-2xl group">
      <video
        ref={videoRef}
        src={props.src}
        class="w-full h-auto max-h-[50vh] sm:max-h-[60vh] md:max-h-[70vh] object-contain cursor-pointer"
        onLoadedMetadata={handleLoadedMetadata}
        onTimeUpdate={handleTimeUpdate}
        onPlay={() => setPlaying(true)}
        onPause={() => setPlaying(false)}
        onEnded={() => {
          setPlaying(false);
          if (props.onEnded) props.onEnded();
        }}
        onClick={toggleControls}
        playsinline
      />
      {/* Controls overlay */}
      <div
        class={`absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent p-2 sm:p-4 transition-opacity duration-300 ${
          controlsVisible() ? 'opacity-100' : 'opacity-0'
        }`}
        onMouseEnter={showControls}
        onMouseLeave={() => {
          // Delay hiding a bit to allow moving to controls
          if (controlsTimeout) clearTimeout(controlsTimeout);
          controlsTimeout = setTimeout(() => {
            setControlsVisible(false);
          }, 1500);
        }}
        // Touch: keep visible when interacting
        onTouchStart={showControls}
      >
        <div class="flex flex-col gap-1 sm:gap-2">
          <div class="flex items-center gap-1 sm:gap-2">
            <span class="text-white text-[10px] sm:text-xs font-mono">{formatTime(currentTime())}</span>
            <input
              type="range"
              min={0}
              max={duration() || 0}
              value={currentTime()}
              onInput={handleSeek}
              class="flex-1 h-1 bg-white/30 rounded-lg appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2 sm:w-3 [&::-webkit-slider-thumb]:h-2 sm:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-cyan-400"
            />
            <span class="text-white text-[10px] sm:text-xs font-mono">{formatTime(duration())}</span>
          </div>
          <div class="flex items-center gap-2 sm:gap-4 text-white">
            <button onClick={togglePlay} class="hover:scale-110 transition p-1">
              {playing() ? <PauseIcon /> : <PlayIcon />}
            </button>
            <div class="flex items-center gap-1">
              <button onClick={toggleMute} class="hover:scale-110 transition p-1">
                {muted() || volume() === 0 ? <MuteIcon /> : <VolumeIcon />}
              </button>
              <input
                type="range"
                min={0}
                max={1}
                step={0.01}
                value={muted() ? 0 : volume()}
                onInput={handleVolumeChange}
                class="w-12 sm:w-16 h-1 bg-white/30 rounded-lg appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-2 sm:w-3 [&::-webkit-slider-thumb]:h-2 sm:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-cyan-400"
              />
            </div>
            <div class="flex-1" />
            <button onClick={toggleFullscreen} class="hover:scale-110 transition p-1">
              {fullscreen() ? <FullscreenExitIcon /> : <FullscreenIcon />}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

// ---------- Detail Page ----------
function Detail() {
  const params = useParams<{ type: string; id: string }>();
  const [detail] = createResource(() => params.type && params.id, () => fetchMediaDetail(params.type, params.id));

  const [selectedEpisode, setSelectedEpisode] = createSignal<Episode | null>(null);

  createEffect(() => {
    const data = detail();
    if (data && data.type === 'series' && data.episodes && data.episodes.length > 0) {
      setSelectedEpisode(data.episodes[0]);
    } else {
      setSelectedEpisode(null);
    }
  });

  const videoSrc = () => {
    const data = detail();
    if (!data) return '';
    if (data.type === 'movie') {
      return data.filePath;
    } else if (data.type === 'series' && selectedEpisode()) {
      return selectedEpisode()!.filePath;
    }
    return '';
  };

  const selectEpisode = (ep: Episode) => {
    setSelectedEpisode(ep);
  };

  return (
    <Suspense fallback={<div class="min-h-screen flex items-center justify-center text-white">جارٍ التحميل...</div>}>
      <div class="relative min-h-screen bg-black text-white overflow-hidden">
        <div class="absolute inset-0">
          <img src={detail()?.poster || 'https://via.placeholder.com/300x450?text=لا+صورة'} class="w-full h-full object-cover scale-110 blur-3xl opacity-30" alt="" />
          <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent" />
        </div>

        <div class="relative z-10 max-w-7xl mx-auto px-3 sm:px-4 lg:px-8 py-16 sm:py-20 md:py-32">
          <div class="flex flex-col lg:flex-row gap-4 sm:gap-6 lg:gap-8 items-start">
            <div class="flex-shrink-0 w-32 sm:w-40 md:w-48 lg:w-64 mx-auto lg:mx-0">
              <img src={detail()?.poster || 'https://via.placeholder.com/300x450?text=لا+صورة'} class="w-full rounded-xl sm:rounded-2xl shadow-2xl border border-white/10" alt={detail()?.title} />
            </div>

            <div class="flex-1 w-full">
              <div class="inline-flex items-center gap-1.5 sm:gap-2 bg-white/10 backdrop-blur-md rounded-full px-2.5 py-0.5 sm:px-3 sm:py-1 text-[10px] sm:text-sm font-medium mb-2 sm:mb-4">
                {detail()?.type === 'movie' ? <MovieIcon /> : <SeriesIcon />}
                {detail()?.type === 'movie' ? 'فيلم' : 'مسلسل'}
              </div>
              <h1 class="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-extrabold tracking-tight mb-1 sm:mb-2">{detail()?.title}</h1>
              <div class="flex flex-wrap items-center gap-2 sm:gap-4 text-gray-300 mt-1 sm:mt-2 mb-3 sm:mb-6 text-xs sm:text-sm md:text-base">
                <Show when={detail()?.year}><span>{detail()?.year}</span></Show>
                <span class="flex items-center gap-0.5 sm:gap-1"><ClockIcon />{detail()?.duration}</span>
                <span>{detail()?.size}</span>
              </div>
              <p class="text-gray-400 leading-relaxed max-w-2xl text-sm sm:text-base md:text-lg">{detail()?.description || 'لا يوجد وصف متاح.'}</p>
              <div class="mt-3 sm:mt-6 flex gap-2 sm:gap-3">
                {detail()?.type === 'movie' && (
                  <a href={detail()?.filePath} class="inline-flex items-center bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-semibold py-1.5 px-3 sm:py-2 sm:px-5 rounded-lg sm:rounded-xl shadow-lg shadow-cyan-500/20 transition transform hover:scale-105 text-xs sm:text-sm">
                    <DownloadIcon /> تحميل
                  </a>
                )}
              </div>
            </div>
          </div>

          <Show when={videoSrc()}>
            <div class="mt-6 sm:mt-8 lg:mt-10">
              <VideoPlayer src={videoSrc()} title={detail()?.title} />
            </div>
          </Show>

          <Show when={detail()?.type === 'series' && detail()?.episodes && detail()!.episodes!.length > 0}>
            <div class="mt-6 sm:mt-8 lg:mt-10">
              <h2 class="text-lg sm:text-xl md:text-2xl font-bold text-white mb-3 sm:mb-4 flex items-center gap-1.5 sm:gap-2">
                <SeriesIcon /> الحلقات
              </h2>
              <div class="grid grid-cols-1 xs:grid-cols-2 sm:grid-cols-2 md:grid-cols-3 gap-2 sm:gap-3">
                <For each={detail()!.episodes}>
                  {(ep) => (
                    <div
                      class={`p-2 sm:p-3 rounded-lg sm:rounded-xl border transition cursor-pointer ${
                        selectedEpisode()?.id === ep.id
                          ? 'border-cyan-400 bg-cyan-400/10 shadow-lg shadow-cyan-400/10'
                          : 'border-white/10 bg-white/5 hover:bg-white/10'
                      }`}
                      onClick={() => selectEpisode(ep)}
                    >
                      <div class="flex items-center gap-1.5 sm:gap-3">
                        <span class="text-[10px] sm:text-sm font-mono text-gray-400">S{String(ep.season).padStart(2, '0')}E{String(ep.episode).padStart(2, '0')}</span>
                        <span class="text-[10px] sm:text-sm text-white truncate">{ep.title}</span>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>
          </Show>
        </div>
      </div>
    </Suspense>
  );
}

// ---------- Pages: Home, Movies, Series, Search ----------
function Home() {
  const [media] = createResource<Media[]>(fetchAllMedia);
  const movies = () => media()?.filter(m => m.type === 'movie') || [];
  const series = () => media()?.filter(m => m.type === 'series') || [];

  return (
    <div class="max-w-7xl mx-auto px-3 sm:px-4 lg:px-8">
      <div class="py-8 sm:py-12 md:py-16 lg:py-24 text-center">
        <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-7xl font-extrabold tracking-tight mb-2 sm:mb-4">
          <span class="bg-gradient-to-r from-cyan-200 to-blue-400 bg-clip-text text-transparent">سينماك</span>
          <br class="sm:hidden" />
          <span class="text-white"> الشخصية</span>
        </h1>
        <p class="text-gray-400 text-sm sm:text-base md:text-lg lg:text-xl max-w-2xl mx-auto px-2">
          شاهد وحمّل مجموعتك من الأفلام والمسلسلات من أي مكان في منزلك.
        </p>
      </div>
      <Suspense fallback={<div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3 sm:gap-4 md:gap-6"><For each={Array(5)}>{() => <CardSkeleton />}</For></div>}>
        <section class="mb-8 sm:mb-12 md:mb-16">
          <div class="flex items-center justify-between mb-3 sm:mb-4 md:mb-6 lg:mb-8">
            <h2 class="text-xl sm:text-2xl md:text-3xl lg:text-4xl font-bold text-white flex items-center gap-1.5 sm:gap-2 md:gap-3"><span class="text-cyan-400"><MovieIcon /></span> أفلام</h2>
            <A href="/movies" class="text-cyan-400 hover:text-cyan-300 text-xs sm:text-sm font-medium transition-all flex items-center gap-0.5 sm:gap-1"><span class="text-base sm:text-lg">←</span> عرض الكل</A>
          </div>
          <div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3 sm:gap-4 md:gap-6"><For each={movies().slice(0, 5)}>{(item) => <MediaCard item={item} type="movie" />}</For></div>
        </section>
        <section class="mb-8 sm:mb-12 md:mb-16">
          <div class="flex items-center justify-between mb-3 sm:mb-4 md:mb-6 lg:mb-8">
            <h2 class="text-xl sm:text-2xl md:text-3xl lg:text-4xl font-bold text-white flex items-center gap-1.5 sm:gap-2 md:gap-3"><span class="text-purple-400"><SeriesIcon /></span> مسلسلات</h2>
            <A href="/series" class="text-purple-400 hover:text-purple-300 text-xs sm:text-sm font-medium transition-all flex items-center gap-0.5 sm:gap-1"><span class="text-base sm:text-lg">←</span> عرض الكل</A>
          </div>
          <div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-3 sm:gap-4 md:gap-6"><For each={series().slice(0, 5)}>{(item) => <MediaCard item={item} type="series" />}</For></div>
        </section>
      </Suspense>
    </div>
  );
}

function Movies() {
  const [movies] = createResource<Media[]>(fetchMovies);
  return (
    <div class="max-w-7xl mx-auto px-3 sm:px-4 lg:px-8">
      <div class="flex items-center gap-2 sm:gap-3 md:gap-4 mb-4 sm:mb-6 md:mb-8 lg:mb-10">
        <div class="p-2 sm:p-3 bg-cyan-400/10 rounded-xl sm:rounded-2xl text-cyan-400"><MovieIcon /></div>
        <div><h1 class="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-extrabold text-white">أفلام</h1><p class="text-gray-400 text-xs sm:text-sm md:text-base mt-0.5 sm:mt-1">تصفح مجموعة أفلامك</p></div>
      </div>
      <Suspense fallback={<div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 sm:gap-4 md:gap-6"><For each={Array(8)}>{() => <CardSkeleton />}</For></div>}>
        <div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 sm:gap-4 md:gap-6"><For each={movies()}>{(m) => <MediaCard item={m} type="movie" />}</For></div>
      </Suspense>
    </div>
  );
}

function Series() {
  const [series] = createResource<Media[]>(fetchSeries);
  return (
    <div class="max-w-7xl mx-auto px-3 sm:px-4 lg:px-8">
      <div class="flex items-center gap-2 sm:gap-3 md:gap-4 mb-4 sm:mb-6 md:mb-8 lg:mb-10">
        <div class="p-2 sm:p-3 bg-purple-400/10 rounded-xl sm:rounded-2xl text-purple-400"><SeriesIcon /></div>
        <div><h1 class="text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-extrabold text-white">مسلسلات</h1><p class="text-gray-400 text-xs sm:text-sm md:text-base mt-0.5 sm:mt-1">تصفح مجموعة مسلسلاتك</p></div>
      </div>
      <Suspense fallback={<div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 sm:gap-4 md:gap-6"><For each={Array(8)}>{() => <CardSkeleton />}</For></div>}>
        <div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 sm:gap-4 md:gap-6"><For each={series()}>{(s) => <MediaCard item={s} type="series" />}</For></div>
      </Suspense>
    </div>
  );
}

function Search() {
  const [searchParams] = useSearchParams<{ q: string }>();
  const query = () => (searchParams.q || '').toLowerCase().trim();
  const [allMedia] = createResource<Media[]>(fetchAllMedia);
  const results = () => {
    const media = allMedia();
    if (!media || !query()) return [];
    return media.filter(item => item.title.toLowerCase().includes(query()));
  };
  return (
    <div class="max-w-7xl mx-auto px-3 sm:px-4 lg:px-8">
      <div class="mb-4 sm:mb-6 md:mb-8 lg:mb-10">
        <h1 class="text-2xl sm:text-3xl md:text-4xl font-extrabold text-white mb-1 sm:mb-2">نتائج البحث</h1>
        <Show when={query()} fallback={<p class="text-gray-400 text-sm sm:text-base">أدخل كلمة بحث للعثور على الوسائط.</p>}>
          <p class="text-gray-400 text-sm sm:text-base">نتائج البحث عن <span class="text-white font-semibold">"{searchParams.q}"</span></p>
        </Show>
      </div>
      <Suspense fallback={<div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 sm:gap-4 md:gap-6"><For each={Array(4)}>{() => <CardSkeleton />}</For></div>}>
        <Show when={results().length > 0} fallback={<div class="text-center py-10 sm:py-16 md:py-20 text-gray-400 text-sm sm:text-base md:text-lg">لا يوجد وسائط تطابق بحثك.</div>}>
          <div class="grid grid-cols-2 xs:grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 sm:gap-4 md:gap-6"><For each={results()}>{(item) => <MediaCard item={item} type={item.type} />}</For></div>
        </Show>
      </Suspense>
    </div>
  );
}

// ---------- Upload Page ----------
function Upload() {
  const [title, setTitle] = createSignal('');
  const [type, setType] = createSignal<'movie' | 'series'>('series');
  const [description, setDescription] = createSignal('');

  const [movieFile, setMovieFile] = createSignal<File | null>(null);

  const [isNewSeries, setIsNewSeries] = createSignal(true);
  const [existingSeriesId, setExistingSeriesId] = createSignal<number | null>(null);

  const [allMedia] = createResource<Media[]>(fetchAllMedia);
  const seriesList = () => allMedia()?.filter(m => m.type === 'series') || [];

  // When switching to "new series", clear the title (fixes issue #2)
  createEffect(() => {
    if (type() === 'series' && isNewSeries()) {
      setTitle('');
    }
  });

  // Auto‑fill title when existing series is selected
  createEffect(() => {
    const id = existingSeriesId();
    if (!isNewSeries() && id !== null) {
      const found = seriesList().find(s => s.id === id);
      if (found) setTitle(found.title);
    }
  });

  const [episodes, setEpisodes] = createSignal<{ id: number; file: File; title: string }[]>([]);
  const [nextId, setNextId] = createSignal(1);

  const [loading, setLoading] = createSignal(false);
  const [success, setSuccess] = createSignal(false);
  const [error, setError] = createSignal('');

  const handleMultiFileSelect = (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    const files = Array.from(input.files);
    const newEpisodes = files.map((file) => ({
      id: nextId(),
      file: file,
      title: file.name.replace(/\.[^/.]+$/, ''),
    }));
    newEpisodes.sort((a, b) => a.file.name.localeCompare(b.file.name));
    setEpisodes([...episodes(), ...newEpisodes]);
    setNextId(prev => prev + files.length);
    input.value = '';
    setError(''); // clear any previous error
  };

  const removeEpisode = (id: number) => {
    setEpisodes(episodes().filter(ep => ep.id !== id));
  };

  const updateEpisodeTitle = (id: number, newTitle: string) => {
    setEpisodes(episodes().map(ep => ep.id === id ? { ...ep, title: newTitle } : ep));
  };

  const moveEpisode = (id: number, direction: 'up' | 'down') => {
    const index = episodes().findIndex(ep => ep.id === id);
    if (index === -1) return;
    const newIndex = direction === 'up' ? index - 1 : index + 1;
    if (newIndex < 0 || newIndex >= episodes().length) return;
    const newEpisodes = [...episodes()];
    [newEpisodes[index], newEpisodes[newIndex]] = [newEpisodes[newIndex], newEpisodes[index]];
    setEpisodes(newEpisodes);
  };

  const sortEpisodes = () => {
    const sorted = [...episodes()].sort((a, b) => a.file.name.localeCompare(b.file.name));
    setEpisodes(sorted);
  };

  const handleMovieFileChange = (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      setMovieFile(input.files[0]);
    }
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setError('');

    // Validation: if series, must have at least one episode
    if (type() === 'series' && episodes().length === 0) {
      setError('يجب إضافة حلقة واحدة على الأقل للمسلسل.');
      return;
    }

    setLoading(true);
    await delay(1000);
    const data: any = {
      title: title(),
      type: type(),
      description: description(),
      ...(type() === 'movie'
        ? { movieFile: movieFile()?.name || 'No file' }
        : {
            isNewSeries: isNewSeries(),
            existingSeriesId: existingSeriesId(),
            episodes: episodes().map((ep, idx) => ({
              number: idx + 1,
              title: ep.title,
              fileName: ep.file.name,
            }))
          }
      )
    };
    console.log('Upload data:', data);
    setLoading(false);
    setSuccess(true);
    setTitle('');
    setDescription('');
    setMovieFile(null);
    setEpisodes([]);
    setNextId(1);
    setIsNewSeries(true);
    setExistingSeriesId(null);
    setTimeout(() => setSuccess(false), 3000);
  };

  return (
    <div class="max-w-3xl mx-auto px-3 sm:px-4 lg:px-6">
      <div class="mb-6 sm:mb-8 md:mb-10 text-center">
        <div class="inline-flex items-center justify-center p-3 sm:p-4 bg-cyan-400/10 rounded-2xl sm:rounded-3xl mb-3 sm:mb-4">
          <span class="text-cyan-400"><UploadIcon /></span>
        </div>
        <h1 class="text-2xl sm:text-3xl md:text-4xl font-extrabold text-white">رفع وسائط جديدة</h1>
        <p class="text-gray-400 text-sm sm:text-base mt-1 sm:mt-2">أضف فيلمًا أو مسلسلًا إلى مكتبتك المنزلية</p>
      </div>

      <div class="backdrop-blur-xl bg-white/5 rounded-2xl sm:rounded-3xl border border-white/10 p-4 sm:p-6 md:p-8 shadow-2xl">
        <form onSubmit={handleSubmit} class="space-y-5 sm:space-y-6 md:space-y-8">
          <div class="flex justify-center">
            <div class="inline-flex bg-white/5 rounded-xl sm:rounded-2xl p-0.5 sm:p-1" role="group">
              <button
                type="button"
                onClick={() => setType('series')}
                class={`px-3 sm:px-4 md:px-6 py-1.5 sm:py-2 md:py-2.5 rounded-lg sm:rounded-xl text-xs sm:text-sm font-medium transition flex items-center gap-1 sm:gap-2 ${
                  type() === 'series' ? 'bg-purple-500/20 text-purple-400 shadow-lg shadow-purple-500/5' : 'text-gray-400 hover:text-white'
                }`}
              >
                <SeriesIcon /> مسلسل
              </button>
              <button
                type="button"
                onClick={() => setType('movie')}
                class={`px-3 sm:px-4 md:px-6 py-1.5 sm:py-2 md:py-2.5 rounded-lg sm:rounded-xl text-xs sm:text-sm font-medium transition flex items-center gap-1 sm:gap-2 ${
                  type() === 'movie' ? 'bg-cyan-500/20 text-cyan-400 shadow-lg shadow-cyan-500/5' : 'text-gray-400 hover:text-white'
                }`}
              >
                <MovieIcon /> فيلم
              </button>
            </div>
          </div>

          <div class="space-y-3 sm:space-y-4 md:space-y-5">
            <div>
              <label class="block text-xs sm:text-sm font-medium text-gray-300 mb-0.5 sm:mb-1">العنوان *</label>
              <input
                type="text"
                value={title()}
                onInput={(e) => setTitle(e.currentTarget.value)}
                required
                placeholder="مثال: Breaking Bad"
                class={`w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-lg sm:rounded-xl py-2 sm:py-3 px-3 sm:px-4 text-sm sm:text-base focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:bg-white/20 transition ${
                  !isNewSeries() && type() === 'series' ? 'opacity-60 cursor-not-allowed' : ''
                }`}
                disabled={!isNewSeries() && type() === 'series'}
              />
            </div>
            <div>
              <label class="block text-xs sm:text-sm font-medium text-gray-300 mb-0.5 sm:mb-1">الوصف (اختياري)</label>
              <textarea
                value={description()}
                onInput={(e) => setDescription(e.currentTarget.value)}
                rows={3}
                placeholder="وصف مختصر (اختياري)..."
                class="w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-lg sm:rounded-xl py-2 sm:py-3 px-3 sm:px-4 text-sm sm:text-base focus:outline-none focus:ring-2 focus:ring-cyan-400 focus:bg-white/20 transition resize-none"
              />
            </div>
          </div>

          <Show when={type() === 'series'}>
            <div class="space-y-3 sm:space-y-4">
              <div class="flex flex-wrap items-center gap-2 sm:gap-4">
                <label class="text-xs sm:text-sm font-medium text-gray-300">نوع المسلسل:</label>
                <div class="inline-flex bg-white/5 rounded-lg sm:rounded-xl p-0.5 sm:p-1">
                  <button
                    type="button"
                    onClick={() => { setIsNewSeries(true); setExistingSeriesId(null); }}
                    class={`px-2 sm:px-3 md:px-4 py-1 sm:py-1.5 rounded-lg text-xs sm:text-sm font-medium transition ${
                      isNewSeries() ? 'bg-cyan-500/20 text-cyan-400' : 'text-gray-400 hover:text-white'
                    }`}
                  >
                    جديد
                  </button>
                  <button
                    type="button"
                    onClick={() => setIsNewSeries(false)}
                    class={`px-2 sm:px-3 md:px-4 py-1 sm:py-1.5 rounded-lg text-xs sm:text-sm font-medium transition ${
                      !isNewSeries() ? 'bg-cyan-500/20 text-cyan-400' : 'text-gray-400 hover:text-white'
                    }`}
                  >
                    موجود
                  </button>
                </div>
              </div>

              <Show when={!isNewSeries()}>
                <div>
                  <label class="block text-xs sm:text-sm font-medium text-gray-300 mb-0.5 sm:mb-1">اختر المسلسل الموجود</label>
                  <select
                    value={existingSeriesId() || ''}
                    onChange={(e) => setExistingSeriesId(Number(e.currentTarget.value) || null)}
                    class="w-full bg-white/10 backdrop-blur-md text-white rounded-lg sm:rounded-xl py-2 sm:py-3 px-3 sm:px-4 text-sm sm:text-base focus:outline-none focus:ring-2 focus:ring-cyan-400"
                  >
                    <option value="" class="bg-gray-800">-- اختر --</option>
                    <For each={seriesList()}>
                      {(s) => <option value={s.id} class="bg-gray-800">{s.title}</option>}
                    </For>
                  </select>
                </div>
              </Show>
            </div>
          </Show>

          <Show when={type() === 'movie'}>
            <div>
              <label class="block text-xs sm:text-sm font-medium text-gray-300 mb-0.5 sm:mb-1">ملف الفيلم</label>
              <div class="flex flex-wrap items-center gap-2 sm:gap-4">
                <input
                  type="file"
                  id="movieFileInput"
                  class="hidden"
                  onChange={handleMovieFileChange}
                  accept="video/*"
                />
                <label
                  for="movieFileInput"
                  class="inline-flex items-center gap-1 sm:gap-2 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1.5 sm:py-2 px-3 sm:px-4 rounded-lg sm:rounded-xl cursor-pointer transition text-xs sm:text-sm"
                >
                  <UploadIcon /> اختر ملف
                </label>
                <span class="text-xs sm:text-sm text-gray-400">
                  {movieFile() ? movieFile()!.name : 'لم يتم اختيار ملف'}
                </span>
              </div>
            </div>
          </Show>

          <Show when={type() === 'series'}>
            <div class="space-y-3 sm:space-y-4">
              <div class="flex flex-wrap items-center justify-between gap-2 sm:gap-3">
                <h2 class="text-base sm:text-lg md:text-xl font-bold text-white flex items-center gap-1.5 sm:gap-2">
                  <SeriesIcon /> الحلقات
                </h2>
                <div class="flex flex-wrap items-center gap-1.5 sm:gap-2">
                  <input
                    type="file"
                    id="multiEpisodeInput"
                    class="hidden"
                    multiple
                    accept="video/*"
                    onChange={handleMultiFileSelect}
                  />
                  <label
                    for="multiEpisodeInput"
                    class="inline-flex items-center gap-0.5 sm:gap-1 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1 sm:py-1.5 px-2 sm:px-3 rounded-lg cursor-pointer transition text-[10px] sm:text-sm"
                  >
                    <UploadIcon /> اختيار ملفات
                  </label>
                  <button
                    type="button"
                    onClick={sortEpisodes}
                    class="inline-flex items-center gap-0.5 sm:gap-1 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1 sm:py-1.5 px-2 sm:px-3 rounded-lg transition text-[10px] sm:text-sm"
                    title="ترتيب الحلقات أبجدياً"
                  >
                    <SortIcon /> ترتيب
                  </button>
                </div>
              </div>

              <Show when={episodes().length > 0} fallback={
                <div class="text-center py-6 sm:py-8 text-gray-500 text-xs sm:text-sm border border-dashed border-white/10 rounded-lg sm:rounded-xl">
                  لا توجد حلقات. استخدم "اختيار ملفات" لرفع عدة حلقات دفعة واحدة.
                </div>
              }>
                <div class="space-y-2 sm:space-y-3 max-h-72 sm:max-h-96 overflow-y-auto p-0.5 sm:p-1">
                  <For each={episodes()}>
                    {(ep, index) => (
                      <div class="bg-white/5 backdrop-blur-sm rounded-lg sm:rounded-xl border border-white/10 p-2 sm:p-3 md:p-4 flex flex-col sm:flex-row gap-2 sm:gap-3 items-start">
                        <div class="flex-1 grid grid-cols-1 xs:grid-cols-2 sm:grid-cols-3 gap-1.5 sm:gap-3 w-full">
                          <div>
                            <span class="text-gray-400 text-[10px] sm:text-sm font-medium">رقم الحلقة</span>
                            <div class="text-white font-semibold text-sm sm:text-base mt-0.5 sm:mt-1">{index() + 1}</div>
                          </div>
                          <div class="sm:col-span-2">
                            <label class="text-[10px] sm:text-xs text-gray-400 mb-0.5 sm:mb-1 block">عنوان الحلقة</label>
                            <input
                              type="text"
                              value={ep.title}
                              onInput={(e) => updateEpisodeTitle(ep.id, e.currentTarget.value)}
                              placeholder="عنوان الحلقة"
                              class="w-full bg-white/10 text-white rounded-lg py-1.5 sm:py-2 px-2 sm:px-3 text-xs sm:text-sm focus:outline-none focus:ring-1 focus:ring-cyan-400"
                            />
                          </div>
                          <div class="hidden sm:block">
                            <span class="text-[10px] sm:text-xs text-gray-400">الملف</span>
                            <div class="text-[10px] sm:text-xs text-gray-300 truncate mt-0.5 sm:mt-1 max-w-24 sm:max-w-32">{ep.file.name}</div>
                          </div>
                        </div>
                        <div class="flex items-center gap-0.5 sm:gap-1 mt-1 sm:mt-0">
                          <button
                            type="button"
                            onClick={() => moveEpisode(ep.id, 'up')}
                            disabled={index() === 0}
                            class="text-gray-400 hover:text-white transition disabled:opacity-30 p-0.5 sm:p-1"
                            title="نقل للأعلى"
                          >
                            <UpArrow />
                          </button>
                          <button
                            type="button"
                            onClick={() => moveEpisode(ep.id, 'down')}
                            disabled={index() === episodes().length - 1}
                            class="text-gray-400 hover:text-white transition disabled:opacity-30 p-0.5 sm:p-1"
                            title="نقل للأسفل"
                          >
                            <DownArrow />
                          </button>
                          <button
                            type="button"
                            onClick={() => removeEpisode(ep.id)}
                            class="text-red-400 hover:text-red-300 transition p-0.5 sm:p-1"
                            title="حذف الحلقة"
                          >
                            <DeleteIcon />
                          </button>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
                <p class="text-[10px] sm:text-xs text-gray-500">
                  يتم ترقيم الحلقات تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب، أو زر "ترتيب" للفرز الأبجدي.
                </p>
              </Show>
            </div>
          </Show>

          {/* Error message */}
          <Show when={error()}>
            <div class="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm text-center">
              {error()}
            </div>
          </Show>

          <button
            type="submit"
            disabled={loading()}
            class="w-full py-2 sm:py-3 px-4 sm:px-6 rounded-lg sm:rounded-xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-sm sm:text-base md:text-lg shadow-lg shadow-cyan-500/20 transition transform hover:scale-[1.02] disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 flex items-center justify-center gap-1.5 sm:gap-2"
          >
            {loading() ? (
              <span class="inline-block w-4 h-4 sm:w-5 sm:h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
            ) : (
              <UploadIcon />
            )}
            {loading() ? 'جارٍ الرفع...' : 'رفع الوسائط'}
          </button>
        </form>

        <Show when={success()}>
          <div class="mt-4 sm:mt-6 p-3 sm:p-4 bg-green-500/10 border border-green-500/30 rounded-lg sm:rounded-xl text-green-400 text-center text-sm sm:text-base animate-fadeIn">
            تم رفع الوسائط بنجاح! ستظهر في المكتبة قريباً.
          </div>
        </Show>
      </div>
    </div>
  );
}

// ---------- Settings Placeholder ----------
function Settings() {
  return (
    <div class="max-w-7xl mx-auto px-3 sm:px-4 lg:px-8 py-16">
      <div class="text-center text-white">
        <h1 class="text-3xl font-bold mb-4">الإعدادات</h1>
        <p class="text-gray-400">سيتم إضافة صفحة الإعدادات قريباً.</p>
      </div>
    </div>
  );
}

// ---------- App Entry ----------
function App() {
  createEffect(() => {
    document.documentElement.dir = 'rtl';
    document.documentElement.lang = 'ar';
  });
  return (
    <Router>
      <Route path="/" component={Layout}>
        <Route path="/" component={Home} />
        <Route path="/movies" component={Movies} />
        <Route path="/series" component={Series} />
        <Route path="/upload" component={Upload} />
        <Route path="/search" component={Search} />
        <Route path="/settings" component={Settings} />
        <Route path="/:type/:id" component={Detail} />
      </Route>
    </Router>
  );
}

export default App;
