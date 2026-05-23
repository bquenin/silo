import { useState } from 'react';
import { LibraryLayout } from './layouts/library-layout';
import { SpotifyLayout } from './layouts/spotify-layout';
import './App.css';

type LayoutKind = 'library' | 'spotify';

function App() {
  const [layout, setLayout] = useState<LayoutKind>('library');

  return (
    <div className="h-screen w-screen flex flex-col bg-bg">
      {/* prototype-only switcher */}
      <div className="absolute top-3 right-3 z-50 flex items-center gap-1 bg-bg-surface/80 backdrop-blur border border-bg-border rounded p-1 text-xs">
        <button
          onClick={() => setLayout('library')}
          className={`px-3 py-1 rounded transition-colors ${
            layout === 'library'
              ? 'bg-accent text-bg font-medium'
              : 'text-fg-muted hover:text-fg'
          }`}
        >
          Library
        </button>
        <button
          onClick={() => setLayout('spotify')}
          className={`px-3 py-1 rounded transition-colors ${
            layout === 'spotify'
              ? 'bg-accent text-bg font-medium'
              : 'text-fg-muted hover:text-fg'
          }`}
        >
          Spotify
        </button>
      </div>
      <div className="flex-1 min-h-0">
        {layout === 'library' ? <LibraryLayout /> : <SpotifyLayout />}
      </div>
    </div>
  );
}

export default App;
