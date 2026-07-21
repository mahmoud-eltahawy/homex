import { Router, Route, A, useParams, useNavigate, useSearchParams } from '@solidjs/router';
import { createResource, createSignal, createEffect, createMemo, Suspense, For, Show, onMount, onCleanup } from 'solid-js';
import type { ParentProps } from 'solid-js';
import type { JSX } from 'solid-js/jsx-runtime';

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

type UploadData =
  | { title: string; type: 'movie'; description: string; movieFile: string }
  | { title: string; type: 'series'; description: string; isNewSeries: boolean; existingSeriesId: number | null; episodes: { number: number; title: string; fileName: string }[] };

const TEST_VIDEO = 'https://www.w3schools.com/html/mov_bbb.mp4';

const mockMovies: Media[] = [
  { id: 1, title: 'Inception', type: 'movie', poster: 'https://picsum.photos/seed/inception/300/450', filePath: TEST_VIDEO, size: '2.1 جيجابايت', description: 'لص يسرق أسرار الشركات من خلال تقنية مشاركة الأحلام.', year: 2010, duration: 'ساعتان و28 دقيقة' },
  { id: 2, title: 'The Matrix', type: 'movie', poster: 'https://picsum.photos/seed/matrix/300/450', filePath: TEST_VIDEO, size: '1.8 جيجابايت', description: 'هاكر كمبيوتر يكتشف حقيقة الواقع.', year: 1999, duration: 'ساعتان و16 دقيقة' },
  { id: 3, title: 'Interstellar', type: 'movie', poster: 'https://picsum.photos/seed/interstellar/300/450', filePath: TEST_VIDEO, size: '3.1 جيجابايت', description: 'فريق من المستكشفين يسافرون عبر ثقب دودي في الفضاء.', year: 2014, duration: 'ساعتان و49 دقيقة' },
  { id: 4, title: 'The Dark Knight', type: 'movie', poster: 'https://picsum.photos/seed/darkknight/300/450', filePath: TEST_VIDEO, size: '2.5 جيجابايت', description: 'عندما يهدد الجوكر مدينة غوثام بالدمار.', year: 2008, duration: 'ساعتان و32 دقيقة' },
  { id: 5, title: 'Pulp Fiction', type: 'movie', poster: 'https://picsum.photos/seed/pulpfiction/300/450', filePath: TEST_VIDEO, size: '1.9 جيجابايت', description: 'تتشابك حياة اثنين من القتلة وملاكم وزوجين من اللصوص.', year: 1994, duration: 'ساعتان و34 دقيقة' },
];

const mockSeries: Media[] = [
  { id: 101, title: 'Breaking Bad', type: 'series', poster: 'https://picsum.photos/seed/breakingbad/300/450', filePath: '/media/series/breakingbad/', size: '45 جيجابايت (5 مواسم)', description: 'مدرس كيمياء يتحول إلى تاجر مخدرات.', year: 2008, duration: '5 مواسم', episodes: [
    { id: 1011, season: 1, episode: 1, title: 'Pilot', filePath: TEST_VIDEO },
    { id: 1012, season: 1, episode: 2, title: 'Cat\'s in the Bag...', filePath: TEST_VIDEO },
    { id: 1013, season: 1, episode: 3, title: '...And the Bag\'s in the River', filePath: TEST_VIDEO },
    { id: 1014, season: 2, episode: 1, title: 'Seven Thirty-Seven', filePath: TEST_VIDEO },
    { id: 1015, season: 2, episode: 2, title: 'Grilled', filePath: TEST_VIDEO },
  ]},
  { id: 102, title: 'Stranger Things', type: 'series', poster: 'https://picsum.photos/seed/strangerthings/300/450', filePath: '/media/series/strangerthings/', size: '32 جيجابايت (4 مواسم)', description: 'مجموعة من الأطفال يكشفون أسرارًا خارقة في بلدتهم.', year: 2016, duration: '4 مواسم', episodes: [
    { id: 1021, season: 1, episode: 1, title: 'Chapter One: Will Byers', filePath: TEST_VIDEO },
    { id: 1022, season: 1, episode: 2, title: 'Chapter Two: The Weirdo on Maple Street', filePath: TEST_VIDEO },
  ]},
  { id: 103, title: 'The Crown', type: 'series', poster: 'https://picsum.photos/seed/thecrown/300/450', filePath: '/media/series/thecrown/', size: '28 جيجابايت (4 مواسم)', description: 'عهد الملكة إليزابيث الثانية.', year: 2016, duration: '4 مواسم', episodes: [
    { id: 1031, season: 1, episode: 1, title: 'Wolferton Splash', filePath: TEST_VIDEO },
  ]},
  { id: 104, title: 'Game of Thrones', type: 'series', poster: 'https://picsum.photos/seed/got/300/450', filePath: '/media/series/got/', size: '68 جيجابايت (8 مواسم)', description: 'عائلات نبيلة تتصارع على السيطرة على ويستروس.', year: 2011, duration: '8 مواسم', episodes: [
    { id: 1041, season: 1, episode: 1, title: 'Winter Is Coming', filePath: TEST_VIDEO },
    { id: 1042, season: 1, episode: 2, title: 'The Kingsroad', filePath: TEST_VIDEO },
  ]},
];

const delay = (ms: number): Promise<void> => new Promise(resolve => setTimeout(resolve, ms));
const fetchMovies = async (): Promise<Media[]> => { await delay(300); return mockMovies; };
const fetchSeries = async (): Promise<Media[]> => { await delay(300); return mockSeries; };
const fetchAllMedia = async (): Promise<Media[]> => { await delay(300); return [...mockMovies, ...mockSeries]; };
const fetchMediaDetail = async (type: string, id: string): Promise<Media | undefined> => {
  await delay(200);
  const all = type === 'movie' ? mockMovies : mockSeries;
  return all.find(m => m.id === Number(id));
};

const Icon = ({ children, className = '' }: { children: JSX.Element; className?: string }) => (
  <svg xmlns="http://www.w3.org/2000/svg" class={`${className} fill-none stroke-current`} viewBox="0 0 24 24" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">{children}</svg>
);
const SearchIcon = () => <Icon className="h-5 w-5"><path d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/></Icon>;
const MovieIcon = () => <Icon className="h-5 w-5"><path d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4M4 20h16a1 1 0 001-1V5a1 1 0 00-1-1H4a1 1 0 00-1 1v14a1 1 0 001 1z"/></Icon>;
const SeriesIcon = () => <Icon className="h-5 w-5"><path d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/></Icon>;
const DownloadIcon = () => <Icon className="h-5 w-5"><path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></Icon>;
const PlayIcon = () => <Icon className="h-6 w-6"><polygon points="5,3 19,12 5,21"/></Icon>;
const PauseIcon = () => <Icon className="h-6 w-6"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></Icon>;
const ClockIcon = () => <Icon className="h-4 w-4"><circle cx="12" cy="12" r="10"/><polyline points="12,6 12,12 16,14"/></Icon>;
const UploadIcon = () => <Icon className="h-6 w-6"><path d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"/></Icon>;
const DeleteIcon = () => <Icon className="h-5 w-5"><path d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/></Icon>;
const UpArrow = () => <Icon className="h-4 w-4"><polyline points="18,15 12,9 6,15"/></Icon>;
const DownArrow = () => <Icon className="h-4 w-4"><polyline points="6,9 12,15 18,9"/></Icon>;
const SortIcon = () => <Icon className="h-5 w-5"><path d="M3 4h13M3 8h9m-9 4h9m5-4v12m0 0l-4-4m4 4l4-4"/></Icon>;
const VolumeIcon = () => <Icon className="h-5 w-5"><path d="M15.536 8.464a5 5 0 010 7.072m2.828-9.9a9 9 0 010 12.728M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z"/></Icon>;
const MuteIcon = () => <Icon className="h-5 w-5"><path d="M5.586 15H4a1 1 0 01-1-1v-4a1 1 0 011-1h1.586l4.707-4.707C10.923 3.663 12 4.109 12 5v14c0 .891-1.077 1.337-1.707.707L5.586 15z" clip-rule="evenodd"/><path d="M17 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2"/></Icon>;
const FullscreenIcon = () => <Icon className="h-5 w-5"><path d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5v-4m0 4h-4m4 0l-5-5"/></Icon>;
const FullscreenExitIcon = () => <Icon className="h-5 w-5"><path d="M9 9V4M9 4H4M9 4l5 5M15 15V20M15 20h5M15 20l-5-5M9 15v5M9 15H4M9 15l5 5M15 9V4M15 4h5M15 4l-5 5"/></Icon>;

function Layout(props: ParentProps): JSX.Element {
  return (
    <div class="flex flex-col min-h-screen bg-[#0a0a0f] text-white font-sans antialiased" dir="rtl">
      <Navbar />
      <main class="flex-1 bg-gradient-to-b from-[#0a0a0f] via-[#12121a] to-[#0a0a0f] pt-20 md:pt-24 lg:pt-28 pb-8 md:pb-12">
        {props.children}
      </main>
      <Footer />
    </div>
  );
}

function Navbar(): JSX.Element {
  const navigate = useNavigate();
  const [searchTerm, setSearchTerm] = createSignal('');
  const [searchOpen, setSearchOpen] = createSignal(false);

  const handleSearch = (e: Event): void => {
    e.preventDefault();
    const term = searchTerm().trim();
    if (term) {
      navigate(`/search?q=${encodeURIComponent(term)}`);
      setSearchOpen(false);
    }
  };

  return (
    <nav class="fixed top-0 start-0 end-0 z-50 backdrop-blur-xl bg-black/60 border-b border-white/[0.06] shadow-2xl shadow-black/50">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex items-center justify-between h-16 md:h-20">
          <A href="/" class="flex items-center gap-2 text-2xl sm:text-3xl md:text-4xl font-black tracking-tighter">
            <span class="bg-gradient-to-r from-cyan-300 to-blue-500 bg-clip-text text-transparent">وسائطي</span>
          </A>
          <div class="hidden md:flex items-center gap-2">
            <A href="/movies" class="px-4 py-2 rounded-2xl text-sm font-medium text-gray-300 hover:text-white hover:bg-white/10 transition-all duration-300 backdrop-blur-sm">أفلام</A>
            <A href="/series" class="px-4 py-2 rounded-2xl text-sm font-medium text-gray-300 hover:text-white hover:bg-white/10 transition-all duration-300 backdrop-blur-sm">مسلسلات</A>
            <div class={`relative me-2 transition-all duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] ${searchOpen() ? 'w-64' : 'w-10'}`}>
              <form onSubmit={handleSearch} class="flex items-center">
                <button type="button" onClick={() => { setSearchOpen(!searchOpen()); if (!searchOpen()) document.getElementById('mainSearch')?.focus(); else document.getElementById('mainSearch')?.blur(); }} class="absolute start-1 top-1/2 -translate-y-1/2 p-1.5 rounded-full text-gray-400 hover:text-white hover:bg-white/10 transition-colors">
                  <SearchIcon />
                </button>
                <input
                  id="mainSearch"
                  type="text"
                  value={searchTerm()}
                  onInput={(e: Event) => setSearchTerm((e.currentTarget as HTMLInputElement).value)}
                  onFocus={() => setSearchOpen(true)}
                  onBlur={() => { if (!searchTerm()) setSearchOpen(false); }}
                  placeholder="ابحث..."
                  class={`w-full bg-white/5 backdrop-blur-xl text-white placeholder-gray-500 rounded-full py-2.5 pe-4 ps-12 text-sm focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/10 transition-all duration-300 ${searchOpen() ? 'opacity-100 scale-100' : 'opacity-0 scale-95 pointer-events-none'}`}
                />
              </form>
            </div>
          </div>
          <div class="md:hidden flex items-center gap-2">
            <form onSubmit={handleSearch} class="relative flex items-center">
              <input type="text" value={searchTerm()} onInput={(e: Event) => setSearchTerm((e.currentTarget as HTMLInputElement).value)} placeholder="ابحث..." class="w-28 sm:w-36 bg-white/10 backdrop-blur-xl text-white placeholder-gray-400 rounded-full py-1.5 pe-3 ps-3 text-xs focus:outline-none focus:ring-1 focus:ring-cyan-400/50" />
              <button type="submit" class="absolute start-1.5 top-1/2 -translate-y-1/2 text-gray-400"><SearchIcon /></button>
            </form>
          </div>
        </div>
        <div class="md:hidden flex gap-1 pb-2">
          <A href="/movies" class="flex-1 text-center py-1.5 rounded-xl text-xs font-medium text-gray-300 hover:text-white hover:bg-white/10 transition">أفلام</A>
          <A href="/series" class="flex-1 text-center py-1.5 rounded-xl text-xs font-medium text-gray-300 hover:text-white hover:bg-white/10 transition">مسلسلات</A>
        </div>
      </div>
    </nav>
  );
}

function Footer(): JSX.Element {
  return (
    <footer class="bg-[#0a0a0f]/90 backdrop-blur-xl border-t border-white/5 mt-auto">
      <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12 md:py-16">
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-8 md:gap-12">
          <div class="space-y-4">
            <A href="/" class="text-2xl font-black tracking-tighter">
              <span class="bg-gradient-to-r from-cyan-300 to-blue-500 bg-clip-text text-transparent">وسائطي</span>
            </A>
            <p class="text-gray-400 text-sm max-w-xs leading-relaxed">
              خادم السينما الشخصي الخاص بك — شاهد، حمّل، واستمتع بمجموعتك في أي وقت.
            </p>
          </div>
          <div>
            <h3 class="text-white font-semibold text-sm mb-4 tracking-wide">تصفح</h3>
            <ul class="space-y-2 text-sm">
              <li><A href="/movies" class="text-gray-400 hover:text-white transition-colors duration-200">أفلام</A></li>
              <li><A href="/series" class="text-gray-400 hover:text-white transition-colors duration-200">مسلسلات</A></li>
              <li><A href="/search" class="text-gray-400 hover:text-white transition-colors duration-200">بحث</A></li>
            </ul>
          </div>
          <div>
            <h3 class="text-white font-semibold text-sm mb-4 tracking-wide">المكتبة</h3>
            <ul class="space-y-2 text-sm">
              <li><A href="/upload" class="text-gray-400 hover:text-white transition-colors duration-200">رفع وسائط</A></li>
              <li><A href="/settings" class="text-gray-400 hover:text-white transition-colors duration-200">الإعدادات</A></li>
              <li><span class="text-gray-500 cursor-default">v1.0.0</span></li>
            </ul>
          </div>
        </div>
        <div class="mt-10 pt-6 border-t border-white/5 text-center text-gray-500 text-xs tracking-wide">
          <p>© 2025 وسائطي. صُنع بكل ❤️ لشبكتك المنزلية.</p>
        </div>
      </div>
    </footer>
  );
}

function MediaCard({ item, type }: { item: Media; type: string }): JSX.Element {
  return (
    <A href={`/${type}/${item.id}`} class="group relative flex flex-col overflow-hidden rounded-2xl bg-[#1a1a24]/80 backdrop-blur-sm border border-white/5 shadow-2xl hover:shadow-cyan-500/20 transition-all duration-500 hover:scale-[1.03] hover:-translate-y-2">
      <div class="aspect-[2/3] relative overflow-hidden">
        <img src={item.poster || 'https://via.placeholder.com/300x450?text=لا+صورة'} alt={item.title} class="w-full h-full object-cover transition-transform duration-700 ease-[cubic-bezier(0.34,1.56,0.64,1)] group-hover:scale-110" loading="lazy" onError={(e: Event) => { (e.currentTarget as HTMLImageElement).src = 'https://via.placeholder.com/300x450?text=لا+صورة'; }} />
        <div class="absolute inset-0 bg-gradient-to-t from-black via-black/30 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 flex flex-col justify-end p-4">
          <div class="transform translate-y-4 group-hover:translate-y-0 transition-transform duration-500">
            <h3 class="text-white font-bold text-lg leading-tight line-clamp-2">{item.title}</h3>
            <div class="flex items-center gap-2 mt-1 text-gray-300 text-sm">
              <Show when={item.year}><span>{item.year}</span></Show>
              <span class="flex items-center"><ClockIcon />{item.duration}</span>
            </div>
          </div>
        </div>
        <div class="absolute top-3 end-3 bg-black/70 backdrop-blur-md rounded-full px-2.5 py-1 text-xs font-bold text-white flex items-center gap-1.5 border border-white/10">
          {item.type === 'movie' ? <MovieIcon /> : <SeriesIcon />}
          {item.type === 'movie' ? 'فيلم' : 'مسلسل'}
        </div>
      </div>
      <div class="p-4 flex flex-col gap-1">
        <h3 class="text-white font-semibold truncate text-sm">{item.title}</h3>
        <div class="flex items-center justify-between text-gray-500 text-xs">
          <span class="flex items-center gap-1"><Show when={item.year}>{item.year} · </Show>{item.size}</span>
          <span class="text-cyan-400 text-xs font-medium opacity-0 group-hover:opacity-100 transition-opacity">← التفاصيل</span>
        </div>
      </div>
    </A>
  );
}

function CardSkeleton(): JSX.Element {
  return (
    <div class="animate-pulse rounded-2xl bg-[#1a1a24]/60 border border-white/5 overflow-hidden shadow-xl">
      <div class="aspect-[2/3] bg-gradient-to-b from-[#2a2a3a] to-[#1a1a24]" />
      <div class="p-4 space-y-2">
        <div class="h-3 bg-[#2a2a3a] rounded w-3/4" />
        <div class="h-2 bg-[#2a2a3a] rounded w-1/2" />
      </div>
    </div>
  );
}

function VideoPlayer(props: { src: string; title?: string; onEnded?: () => void }): JSX.Element {
  let videoRef: HTMLVideoElement | undefined;
  let controlsTimeout: number | undefined;

  const [playing, setPlaying] = createSignal(false);
  const [currentTime, setCurrentTime] = createSignal(0);
  const [duration, setDuration] = createSignal(0);
  const [volume, setVolume] = createSignal(1);
  const [lastVolume, setLastVolume] = createSignal(1); // stores last non-zero volume
  const [muted, setMuted] = createSignal(false);
  const [fullscreen, setFullscreen] = createSignal(false);
  const [controlsVisible, setControlsVisible] = createSignal(true);

  const startHideTimer = (): void => {
    if (controlsTimeout) clearTimeout(controlsTimeout);
    controlsTimeout = setTimeout(() => setControlsVisible(false), 3000);
  };

  const showControls = (): void => {
    setControlsVisible(true);
    startHideTimer();
  };

  const toggleControls = (): void => {
    if (controlsVisible()) {
      setControlsVisible(false);
      if (controlsTimeout) clearTimeout(controlsTimeout);
    } else showControls();
  };

  const handleUserInteraction = (): void => showControls();

  onCleanup(() => { if (controlsTimeout) clearTimeout(controlsTimeout); });

  const handleLoadedMetadata = (): void => { if (videoRef) setDuration(videoRef.duration); };
  const handleTimeUpdate = (): void => { if (videoRef) setCurrentTime(videoRef.currentTime); };

  const togglePlay = (): void => {
    if (!videoRef) return;
    if (playing()) videoRef.pause();
    else videoRef.play();
    // State is updated by onPlay / onPause events
    handleUserInteraction();
  };

  const handleSeek = (e: Event): void => {
    const input = e.currentTarget as HTMLInputElement;
    const val = parseFloat(input.value);
    if (videoRef && !isNaN(val)) {
      videoRef.currentTime = val;
      setCurrentTime(val);
    }
    handleUserInteraction();
  };

  const handleVolumeChange = (e: Event): void => {
    const input = e.currentTarget as HTMLInputElement;
    const val = parseFloat(input.value);
    if (!isNaN(val) && videoRef) {
      setVolume(val);
      videoRef.volume = val;
      videoRef.muted = val === 0;
      setMuted(val === 0);
      if (val > 0) setLastVolume(val);
    }
    handleUserInteraction();
  };

  const toggleMute = (): void => {
    if (!videoRef) return;
    if (muted()) {
      videoRef.muted = false;
      const restore = lastVolume() || 0.5;
      videoRef.volume = restore;
      setVolume(restore);
      setMuted(false);
    } else {
      setLastVolume(volume() || 0.5);
      videoRef.muted = true;
      setMuted(true);
    }
    handleUserInteraction();
  };

  const toggleFullscreen = (): void => {
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
    const handleFullscreenChange = (): void => {
      setFullscreen(!!document.fullscreenElement);
    };
    document.addEventListener('fullscreenchange', handleFullscreenChange);
    showControls();
    return () => {
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
      if (controlsTimeout) clearTimeout(controlsTimeout);
    };
  });

  // Reload video when src changes (episode / movie switch)
  createEffect(() => {
    if (videoRef && props.src) {
      videoRef.src = props.src;
      videoRef.load();
      setPlaying(false);
      setCurrentTime(0);
      setDuration(0);
    }
  });

  const formatTime = (time: number): string => {
    if (isNaN(time)) return '00:00';
    const mins = Math.floor(time / 60);
    const secs = Math.floor(time % 60);
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  };

  return (
    <div dir="ltr" class="relative bg-black rounded-2xl overflow-hidden shadow-2xl shadow-black/50 group">
      <video
        ref={videoRef}
        title={props.title}
        class="w-full h-auto max-h-[60vh] md:max-h-[70vh] object-contain cursor-pointer"
        onLoadedMetadata={handleLoadedMetadata}
        onTimeUpdate={handleTimeUpdate}
        onPlay={() => setPlaying(true)}
        onPause={() => setPlaying(false)}
        onEnded={() => { setPlaying(false); if (props.onEnded) props.onEnded(); }}
        onClick={toggleControls}
        playsinline
      />
      <div
        class={`absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/90 via-black/50 to-transparent p-3 sm:p-5 transition-opacity duration-300 ${controlsVisible() ? 'opacity-100' : 'opacity-0'}`}
        onMouseEnter={showControls}
        onMouseLeave={() => { if (controlsTimeout) clearTimeout(controlsTimeout); controlsTimeout = setTimeout(() => setControlsVisible(false), 1500); }}
        onTouchStart={showControls}
      >
        <div class="flex flex-col gap-2">
          <div class="flex items-center gap-2">
            <span class="text-white text-xs font-mono">{formatTime(currentTime())}</span>
            <input
              type="range"
              min={0}
              max={duration() || 0}
              value={currentTime()}
              onInput={handleSeek}
              class="flex-1 h-1.5 bg-white/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-cyan-400 [&::-webkit-slider-thumb]:shadow-lg [&::-webkit-slider-thumb]:shadow-cyan-400/30"
            />
            <span class="text-white text-xs font-mono">{formatTime(duration())}</span>
          </div>
          <div class="flex items-center gap-4 text-white">
            <button onClick={togglePlay} class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10">
              {playing() ? <PauseIcon /> : <PlayIcon />}
            </button>
            <div class="flex items-center gap-2">
              <button onClick={toggleMute} class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10">
                {muted() || volume() === 0 ? <MuteIcon /> : <VolumeIcon />}
              </button>
              <input
                type="range"
                min={0}
                max={1}
                step={0.01}
                value={muted() ? 0 : volume()}
                onInput={handleVolumeChange}
                class="w-16 sm:w-20 h-1.5 bg-white/20 rounded-full appearance-none cursor-pointer [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-3 [&::-webkit-slider-thumb]:h-3 [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-cyan-400"
              />
            </div>
            <div class="flex-1" />
            <button onClick={toggleFullscreen} class="hover:scale-110 transition-transform duration-200 p-1 rounded-full hover:bg-white/10">
              {fullscreen() ? <FullscreenExitIcon /> : <FullscreenIcon />}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function Detail(): JSX.Element {
  const params = useParams<{ type: string; id: string }>();
  const [detail] = createResource(() => params.type && params.id, () => fetchMediaDetail(params.type, params.id));
  const [selectedEpisodeSignal, setSelectedEpisodeSignal] = createSignal<Episode | null>(null);

  createEffect(() => {
    const data = detail();
    if (data?.type === 'series' && data.episodes && data.episodes.length > 0) {
      setSelectedEpisodeSignal(data.episodes[0]);
    } else {
      setSelectedEpisodeSignal(null);
    }
  });

  const videoSrc = createMemo(() => {
    const data = detail();
    if (!data) return '';
    if (data.type === 'movie') return data.filePath;
    if (data.type === 'series' && selectedEpisodeSignal()) return selectedEpisodeSignal()!.filePath;
    return '';
  });

  return (
    <Suspense fallback={<div class="min-h-screen flex items-center justify-center text-white text-lg">جارٍ التحميل...</div>}>
      <div class="relative min-h-screen bg-black text-white overflow-hidden">
        <div class="absolute inset-0">
          <img src={detail()?.poster || 'https://via.placeholder.com/300x450?text=لا+صورة'} class="w-full h-full object-cover scale-110 blur-3xl opacity-20" alt="" />
          <div class="absolute inset-0 bg-gradient-to-t from-black via-black/70 to-transparent" />
        </div>
        <div class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-20 md:py-32">
          <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
            <div class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0">
              <img src={detail()?.poster || 'https://via.placeholder.com/300x450?text=لا+صورة'} class="w-full rounded-2xl shadow-2xl border border-white/10" alt={detail()?.title} />
            </div>
            <div class="flex-1 w-full">
              <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
                {detail()?.type === 'movie' ? <MovieIcon /> : <SeriesIcon />}
                {detail()?.type === 'movie' ? 'فيلم' : 'مسلسل'}
              </div>
              <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">{detail()?.title}</h1>
              <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
                <Show when={detail()?.year}><span>{detail()?.year}</span></Show>
                <span class="flex items-center gap-1"><ClockIcon />{detail()?.duration}</span>
                <span>{detail()?.size}</span>
              </div>
              <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">{detail()?.description || 'لا يوجد وصف متاح.'}</p>
              <div class="mt-6 flex gap-3">
                {detail()?.type === 'movie' && (
                  <a href={detail()?.filePath} class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm">
                    <DownloadIcon /> تحميل
                  </a>
                )}
              </div>
            </div>
          </div>
          <Show when={videoSrc()}>
            <div class="mt-10">
              <VideoPlayer src={videoSrc()} title={detail()?.title} />
            </div>
          </Show>
          <Show when={detail()?.type === 'series' && detail()?.episodes && detail()!.episodes!.length > 0}>
            <div class="mt-10">
              <h2 class="text-xl sm:text-2xl font-bold text-white mb-4 flex items-center gap-2">
                <SeriesIcon /> الحلقات
              </h2>
              <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
                <For each={detail()!.episodes}>
                  {(ep: Episode) => (
                    <div
                      class={`p-3 rounded-xl border transition-all cursor-pointer backdrop-blur-sm ${
                        selectedEpisodeSignal()?.id === ep.id
                          ? 'border-cyan-400 bg-cyan-400/10 shadow-lg shadow-cyan-400/10'
                          : 'border-white/10 bg-white/5 hover:bg-white/10 hover:border-white/20'
                      }`}
                      onClick={() => setSelectedEpisodeSignal(ep)}
                    >
                      <div class="flex items-center gap-3">
                        <span class="text-sm font-mono text-gray-400">S{String(ep.season).padStart(2, '0')}E{String(ep.episode).padStart(2, '0')}</span>
                        <span class="text-sm text-white truncate">{ep.title}</span>
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

function Home(): JSX.Element {
  const [media] = createResource<Media[]>(fetchAllMedia);
  const movies = createMemo(() => media()?.filter(m => m.type === 'movie') || []);
  const series = createMemo(() => media()?.filter(m => m.type === 'series') || []);

  return (
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="py-12 sm:py-16 md:py-20 lg:py-24 text-center">
        <h1 class="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-black tracking-tight leading-[1.1]">
          <span class="bg-gradient-to-r from-cyan-200 via-blue-300 to-indigo-400 bg-clip-text text-transparent">سينماك</span>
          <br class="sm:hidden" />
          <span class="text-white"> الشخصية</span>
        </h1>
        <p class="text-gray-400 text-base sm:text-lg md:text-xl max-w-2xl mx-auto mt-4 leading-relaxed">
          شاهد وحمّل مجموعتك من الأفلام والمسلسلات من أي مكان في منزلك.
        </p>
      </div>
      <Suspense fallback={<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-6"><For each={Array(5)}>{() => <CardSkeleton />}</For></div>}>
        <section class="mb-12 md:mb-16">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-2xl sm:text-3xl md:text-4xl font-black text-white flex items-center gap-3"><span class="text-cyan-400"><MovieIcon /></span> أفلام</h2>
            <A href="/movies" class="text-cyan-400 hover:text-cyan-300 text-sm font-medium transition-all flex items-center gap-1 group"><span class="text-lg group-hover:translate-x-1 transition-transform">←</span> عرض الكل</A>
          </div>
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-6">
            <For each={movies().slice(0, 5)}>{(item: Media) => <MediaCard item={item} type="movie" />}</For>
          </div>
        </section>
        <section class="mb-12 md:mb-16">
          <div class="flex items-center justify-between mb-6">
            <h2 class="text-2xl sm:text-3xl md:text-4xl font-black text-white flex items-center gap-3"><span class="text-purple-400"><SeriesIcon /></span> مسلسلات</h2>
            <A href="/series" class="text-purple-400 hover:text-purple-300 text-sm font-medium transition-all flex items-center gap-1 group"><span class="text-lg group-hover:translate-x-1 transition-transform">←</span> عرض الكل</A>
          </div>
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-6">
            <For each={series().slice(0, 5)}>{(item: Media) => <MediaCard item={item} type="series" />}</For>
          </div>
        </section>
      </Suspense>
    </div>
  );
}

function Movies(): JSX.Element {
  const [movies] = createResource<Media[]>(fetchMovies);
  return (
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex items-center gap-4 mb-6 md:mb-8">
        <div class="p-3 bg-cyan-400/10 rounded-2xl text-cyan-400"><MovieIcon /></div>
        <div><h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">أفلام</h1><p class="text-gray-400 text-sm md:text-base mt-0.5">تصفح مجموعة أفلامك</p></div>
      </div>
      <Suspense fallback={<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6"><For each={Array(8)}>{() => <CardSkeleton />}</For></div>}>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
          <For each={movies()}>{(m: Media) => <MediaCard item={m} type="movie" />}</For>
        </div>
      </Suspense>
    </div>
  );
}

function Series(): JSX.Element {
  const [series] = createResource<Media[]>(fetchSeries);
  return (
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex items-center gap-4 mb-6 md:mb-8">
        <div class="p-3 bg-purple-400/10 rounded-2xl text-purple-400"><SeriesIcon /></div>
        <div><h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">مسلسلات</h1><p class="text-gray-400 text-sm md:text-base mt-0.5">تصفح مجموعة مسلسلاتك</p></div>
      </div>
      <Suspense fallback={<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6"><For each={Array(8)}>{() => <CardSkeleton />}</For></div>}>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
          <For each={series()}>{(s: Media) => <MediaCard item={s} type="series" />}</For>
        </div>
      </Suspense>
    </div>
  );
}

function Search(): JSX.Element {
  const [searchParams] = useSearchParams<{ q: string }>();
  const query = () => (searchParams.q || '').toLowerCase().trim();
  const [allMedia] = createResource<Media[]>(fetchAllMedia);
  const results = createMemo(() => {
    const media = allMedia();
    if (!media || !query()) return [];
    return media.filter(item => item.title.toLowerCase().includes(query()));
  });
  return (
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-6 md:mb-8">
        <h1 class="text-3xl sm:text-4xl font-black text-white mb-1">نتائج البحث</h1>
        <Show when={query()} fallback={<p class="text-gray-400 text-sm sm:text-base">أدخل كلمة بحث للعثور على الوسائط.</p>}>
          <p class="text-gray-400 text-sm sm:text-base">
            نتائج البحث عن <span class="text-white font-semibold">{`"${searchParams.q}"`}</span>
          </p>
        </Show>
      </div>
      <Suspense fallback={<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6"><For each={Array(4)}>{() => <CardSkeleton />}</For></div>}>
        <Show when={results().length > 0} fallback={<div class="text-center py-16 text-gray-400 text-sm sm:text-base">لا يوجد وسائط تطابق بحثك.</div>}>
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
            <For each={results()}>{(item: Media) => <MediaCard item={item} type={item.type} />}</For>
          </div>
        </Show>
      </Suspense>
    </div>
  );
}

function Upload(): JSX.Element {
  const [title, setTitle] = createSignal('');
  const [type, setType] = createSignal<'movie' | 'series'>('series');
  const [description, setDescription] = createSignal('');
  const [movieFile, setMovieFile] = createSignal<File | null>(null);
  const [isNewSeries, setIsNewSeries] = createSignal(true);
  const [existingSeriesId, setExistingSeriesId] = createSignal<number | null>(null);
  const [allMedia] = createResource<Media[]>(fetchAllMedia);
  const seriesList = createMemo(() => allMedia()?.filter(m => m.type === 'series') || []);
  const [episodes, setEpisodes] = createSignal<{ id: number; file: File; title: string }[]>([]);
  const [nextId, setNextId] = createSignal(1);
  const [loading, setLoading] = createSignal(false);
  const [success, setSuccess] = createSignal(false);
  const [error, setError] = createSignal('');

  createEffect(() => {
    if (type() === 'series' && isNewSeries()) setTitle('');
  });

  createEffect(() => {
    const id = existingSeriesId();
    if (!isNewSeries() && id !== null) {
      const found = seriesList().find(s => s.id === id);
      if (found) setTitle(found.title);
    }
  });

  const handleMultiFileSelect = (e: Event): void => {
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
    setError('');
  };

  const removeEpisode = (id: number): void => {
    setEpisodes(episodes().filter(ep => ep.id !== id));
  };

  const updateEpisodeTitle = (id: number, newTitle: string): void => {
    setEpisodes(episodes().map(ep => ep.id === id ? { ...ep, title: newTitle } : ep));
  };

  const moveEpisode = (id: number, direction: 'up' | 'down'): void => {
    const index = episodes().findIndex(ep => ep.id === id);
    if (index === -1) return;
    const newIndex = direction === 'up' ? index - 1 : index + 1;
    if (newIndex < 0 || newIndex >= episodes().length) return;
    const newEpisodes = [...episodes()];
    [newEpisodes[index], newEpisodes[newIndex]] = [newEpisodes[newIndex], newEpisodes[index]];
    setEpisodes(newEpisodes);
  };

  const sortEpisodes = (): void => {
    const sorted = [...episodes()].sort((a, b) => a.file.name.localeCompare(b.file.name));
    setEpisodes(sorted);
  };

  const handleMovieFileChange = (e: Event): void => {
    const input = e.currentTarget as HTMLInputElement;
    if (input.files && input.files.length > 0) setMovieFile(input.files[0]);
  };

  const handleSubmit = async (e: Event): Promise<void> => {
    e.preventDefault();
    setError('');
    if (type() === 'series' && episodes().length === 0) {
      setError('يجب إضافة حلقة واحدة على الأقل للمسلسل.');
      return;
    }
    setLoading(true);
    await delay(1000);
    let data: UploadData;
    if (type() === 'movie') {
      data = { title: title(), type: 'movie', description: description(), movieFile: movieFile()?.name || 'No file' };
    } else {
      data = {
        title: title(),
        type: 'series',
        description: description(),
        isNewSeries: isNewSeries(),
        existingSeriesId: existingSeriesId(),
        episodes: episodes().map((ep, idx) => ({ number: idx + 1, title: ep.title, fileName: ep.file.name })),
      };
    }
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
    <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="mb-8 md:mb-10 text-center">
        <div class="inline-flex items-center justify-center p-4 bg-cyan-400/10 rounded-3xl mb-4">
          <span class="text-cyan-400"><UploadIcon /></span>
        </div>
        <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">رفع وسائط جديدة</h1>
        <p class="text-gray-400 text-sm sm:text-base mt-2">أضف فيلمًا أو مسلسلًا إلى مكتبتك المنزلية</p>
      </div>
      <div class="backdrop-blur-xl bg-white/5 rounded-3xl border border-white/10 p-6 md:p-8 shadow-2xl">
        <form onSubmit={handleSubmit} class="space-y-6 md:space-y-8">
          <div class="flex justify-center">
            <div class="inline-flex bg-white/5 rounded-2xl p-1" role="group">
              <button
                type="button"
                onClick={() => setType('series')}
                class={`px-4 sm:px-6 py-2 rounded-xl text-sm font-medium transition flex items-center gap-2 ${
                  type() === 'series' ? 'bg-purple-500/20 text-purple-400 shadow-lg shadow-purple-500/10' : 'text-gray-400 hover:text-white'
                }`}
              >
                <SeriesIcon /> مسلسل
              </button>
              <button
                type="button"
                onClick={() => setType('movie')}
                class={`px-4 sm:px-6 py-2 rounded-xl text-sm font-medium transition flex items-center gap-2 ${
                  type() === 'movie' ? 'bg-cyan-500/20 text-cyan-400 shadow-lg shadow-cyan-500/10' : 'text-gray-400 hover:text-white'
                }`}
              >
                <MovieIcon /> فيلم
              </button>
            </div>
          </div>

          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1.5">العنوان *</label>
              <input
                type="text"
                value={title()}
                onInput={(e: Event) => setTitle((e.currentTarget as HTMLInputElement).value)}
                required
                placeholder="مثال: Breaking Bad"
                class={`w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition ${
                  !isNewSeries() && type() === 'series' ? 'opacity-60 cursor-not-allowed' : ''
                }`}
                disabled={!isNewSeries() && type() === 'series'}
              />
            </div>
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1.5">الوصف (اختياري)</label>
              <textarea
                value={description()}
                onInput={(e: Event) => setDescription((e.currentTarget as HTMLInputElement).value)}
                rows={3}
                placeholder="وصف مختصر (اختياري)..."
                class="w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition resize-none"
              />
            </div>
          </div>

          <Show when={type() === 'series'}>
            <div class="space-y-4">
              <div class="flex flex-wrap items-center gap-4">
                <label class="text-sm font-medium text-gray-300">نوع المسلسل:</label>
                <div class="inline-flex bg-white/5 rounded-xl p-0.5">
                  <button
                    type="button"
                    onClick={() => { setIsNewSeries(true); setExistingSeriesId(null); }}
                    class={`px-3 py-1.5 rounded-lg text-sm font-medium transition ${
                      isNewSeries() ? 'bg-cyan-500/20 text-cyan-400' : 'text-gray-400 hover:text-white'
                    }`}
                  >
                    جديد
                  </button>
                  <button
                    type="button"
                    onClick={() => setIsNewSeries(false)}
                    class={`px-3 py-1.5 rounded-lg text-sm font-medium transition ${
                      !isNewSeries() ? 'bg-cyan-500/20 text-cyan-400' : 'text-gray-400 hover:text-white'
                    }`}
                  >
                    موجود
                  </button>
                </div>
              </div>
              <Show when={!isNewSeries()}>
                <div>
                  <label class="block text-sm font-medium text-gray-300 mb-1.5">اختر المسلسل الموجود</label>
                  <select
                    value={existingSeriesId() || ''}
                    onChange={(e: Event) => setExistingSeriesId(Number((e.currentTarget as HTMLSelectElement).value) || null)}
                    class="w-full bg-white/10 backdrop-blur-md text-white rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50"
                  >
                    <option value="" class="bg-gray-800">-- اختر --</option>
                    <For each={seriesList()}>{(s: Media) => <option value={s.id} class="bg-gray-800">{s.title}</option>}</For>
                  </select>
                </div>
              </Show>
            </div>
          </Show>

          <Show when={type() === 'movie'}>
            <div>
              <label class="block text-sm font-medium text-gray-300 mb-1.5">ملف الفيلم</label>
              <div class="flex flex-wrap items-center gap-4">
                <input type="file" id="movieFileInput" class="hidden" onChange={handleMovieFileChange} accept="video/*" />
                <label for="movieFileInput" class="inline-flex items-center gap-2 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-2 px-5 rounded-xl cursor-pointer transition text-sm">
                  <UploadIcon /> اختر ملف
                </label>
                <span class="text-sm text-gray-400">{movieFile() ? movieFile()!.name : 'لم يتم اختيار ملف'}</span>
              </div>
            </div>
          </Show>

          <Show when={type() === 'series'}>
            <div class="space-y-4">
              <div class="flex flex-wrap items-center justify-between gap-3">
                <h2 class="text-lg font-bold text-white flex items-center gap-2"><SeriesIcon /> الحلقات</h2>
                <div class="flex flex-wrap items-center gap-2">
                  <input type="file" id="multiEpisodeInput" class="hidden" multiple accept="video/*" onChange={handleMultiFileSelect} />
                  <label for="multiEpisodeInput" class="inline-flex items-center gap-1.5 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1.5 px-3 rounded-lg cursor-pointer transition text-sm">
                    <UploadIcon /> اختيار ملفات
                  </label>
                  <button type="button" onClick={sortEpisodes} class="inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1.5 px-3 rounded-lg transition text-sm">
                    <SortIcon /> ترتيب
                  </button>
                </div>
              </div>
              <Show when={episodes().length > 0} fallback={
                <div class="text-center py-8 text-gray-500 text-sm border border-dashed border-white/10 rounded-xl">
                  لا توجد حلقات. استخدم "اختيار ملفات" لرفع عدة حلقات دفعة واحدة.
                </div>
              }>
                <div class="space-y-3 max-h-80 overflow-y-auto p-1">
                  <For each={episodes()}>
                    {(ep: { id: number; file: File; title: string }, index) => (
                      <div class="bg-white/5 backdrop-blur-sm rounded-xl border border-white/10 p-4 flex flex-col sm:flex-row gap-3 items-start">
                        <div class="flex-1 grid grid-cols-1 sm:grid-cols-3 gap-3 w-full">
                          <div>
                            <span class="text-gray-400 text-sm font-medium">رقم الحلقة</span>
                            <div class="text-white font-semibold mt-0.5">{index() + 1}</div>
                          </div>
                          <div class="sm:col-span-2">
                            <label class="text-xs text-gray-400 mb-0.5 block">عنوان الحلقة</label>
                            <input
                              type="text"
                              value={ep.title}
                              onInput={(e: Event) => updateEpisodeTitle(ep.id, (e.currentTarget as HTMLInputElement).value)}
                              placeholder="عنوان الحلقة"
                              class="w-full bg-white/10 text-white rounded-lg py-1.5 px-3 text-sm focus:outline-none focus:ring-1 focus:ring-cyan-400"
                            />
                          </div>
                          <div class="hidden sm:block">
                            <span class="text-xs text-gray-400">الملف</span>
                            <div class="text-xs text-gray-300 truncate mt-0.5 max-w-32">{ep.file.name}</div>
                          </div>
                        </div>
                        <div class="flex items-center gap-1 mt-1 sm:mt-0">
                          <button onClick={() => moveEpisode(ep.id, 'up')} disabled={index() === 0} class="text-gray-400 hover:text-white transition disabled:opacity-30 p-1" title="نقل للأعلى"><UpArrow /></button>
                          <button onClick={() => moveEpisode(ep.id, 'down')} disabled={index() === episodes().length - 1} class="text-gray-400 hover:text-white transition disabled:opacity-30 p-1" title="نقل للأسفل"><DownArrow /></button>
                          <button onClick={() => removeEpisode(ep.id)} class="text-red-400 hover:text-red-300 transition p-1" title="حذف الحلقة"><DeleteIcon /></button>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
                <p class="text-xs text-gray-500">يتم ترقيم الحلقات تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب، أو زر "ترتيب" للفرز الأبجدي.</p>
              </Show>
            </div>
          </Show>

          <Show when={error()}>
            <div class="p-3 bg-red-500/10 border border-red-500/30 rounded-xl text-red-400 text-sm text-center">{error()}</div>
          </Show>

          <button
            type="submit"
            disabled={loading()}
            class="w-full py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-base shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 flex items-center justify-center gap-2"
          >
            {loading() ? <span class="inline-block w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></span> : <UploadIcon />}
            {loading() ? 'جارٍ الرفع...' : 'رفع الوسائط'}
          </button>
        </form>
        <Show when={success()}>
          <div class="mt-6 p-4 bg-green-500/10 border border-green-500/30 rounded-xl text-green-400 text-center text-sm animate-fadeIn">
            تم رفع الوسائط بنجاح! ستظهر في المكتبة قريباً.
          </div>
        </Show>
      </div>
    </div>
  );
}

function Settings(): JSX.Element {
  return (
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
      <div class="text-center text-white">
        <h1 class="text-4xl font-black mb-4">الإعدادات</h1>
        <p class="text-gray-400">سيتم إضافة صفحة الإعدادات قريباً.</p>
      </div>
    </div>
  );
}

function App(): JSX.Element {
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
