import React, { useState, useEffect, useCallback, useRef, type FC } from 'react';
import { decode } from '@msgpack/msgpack';

/* MessagePack type */
type MessagePack = {
  pcm: Uint8Array;
  bpm: number;
};

/* connection status types */
type WebSocketReadyState = 'idle' | 'connecting' | 'connected' | 'disconnected';

/* error type */
type WebSocketError = 'decode' | 'play' | 'websocket' | 'invalid_audio_info';

/* AudioInfo type */
type AudioInfo = {
  channel: number;
  sampleRate: number;
  bitsPerSample: number;
  pcmFormat: string;
};

/* useWebSocket return type */
type UseWebSocketHook = {
  audioInfoState: AudioInfo | null;
  bpmState: number;
  readyState: WebSocketReadyState;
  error: WebSocketError | null;
  connect: () => void;
  disconnect: () => void;
};

// custom hook: useWebSocket
const useWebSocket = (url: string): UseWebSocketHook => {
  //* WebSocket *//
  // WebSocket instance
  const ws = useRef<WebSocket | null>(null);
  // connection state
  const [readyState, setReadyState] = useState<WebSocketReadyState>('idle');
  // error state
  const [error, setError] = useState<WebSocketError | null>(null);

  //* Audio *//
  // AudioInfo state
  const [audioInfoState, setAudioInfoState] = useState<AudioInfo | null>(null);
  const audioInfoRef = useRef<AudioInfo | null>(null);
  // AudioContext instance
  const audioContext = useRef<AudioContext | null>(null);

  //* Animation *//
  const [bpmState, setBpmState] = useState<number>(0);

  // event listeners setup function
  const setupEventListeners = useCallback(() => {
    // do not setup event listeners if WebSocket is not initialized
    if (ws.current == null) {
      return;
    }

    ws.current.binaryType = 'arraybuffer';

    ws.current.onopen = () => {
      console.log('WebSocket connection established');
      setReadyState('connected');
      //* step1: send open message *//
      ws.current?.send('open');
    };

    ws.current.onmessage = async (event: MessageEvent) => {
      /* binary data */
      if (event.data instanceof ArrayBuffer) {
        let message: MessagePack | null = null;
        //* step6: received and decode MessagePack data *//
        try {
          message = decode(event.data) as MessagePack;
          console.log('Received MessagePack data:', message);
          //* step7: set BPM *//
          setBpmState(message.bpm);
        } catch (error) {
          console.error('Failed to decode MessagePack:', error);
          setError('decode');
        }
        //* step8: play PCM data *//
        // try {
        //     if (audioContext.current && message) {
        //         const pcmBuffer: ArrayBuffer = message.pcm.buffer;
        //         const audioBuffer = await audioContext.current.decodeAudioData(pcmBuffer);
        //         const audioSource = audioContext.current.createBufferSource();
        //         if (audioSource && audioBuffer) {
        //             audioSource.buffer = audioBuffer;
        //             audioSource.connect(audioContext.current.destination);
        //             audioSource.start();
        //         } else {
        //             console.error('AudioContext or AudioBufferSourceNode is not available');
        //             setError('play');
        //         }
        //     } else {
        //         console.error('No message received or message is null');
        //         setError('play');
        //     }
        // } catch (error) {
        //     console.error('Failed to play PCM data:', error);
        //     setError('play');
        // }
      }
      /* test data */
      if (typeof event.data === 'string') {
        console.log('Received text data:', event.data);
        //* step2: parse AudioInfo *//
        /* Received data format:
            <channels> <sample_rate> <bits_per_sample> <pcm_format>
        */
        const parts = event.data.split(' ');
        if (parts.length === 4) {
          const audioInfo = {
            channel: parseInt(parts[0], 10),
            sampleRate: parseInt(parts[1], 10),
            bitsPerSample: parseInt(parts[2], 10),
            pcmFormat: parts[3],
          };
          if (
            !isNaN(audioInfo.channel) &&
            !isNaN(audioInfo.sampleRate) &&
            !isNaN(audioInfo.bitsPerSample) &&
            audioInfo.pcmFormat
          ) {
            //* step3: set AudioInfo state *//
            setAudioInfoState(audioInfo);
            audioInfoRef.current = audioInfo;
            //* step4: create AudioContext *//
            // NOTE: https://developer.mozilla.org/ja/docs/Web/API/AudioContext/AudioContext
            audioContext.current = new AudioContext({
              latencyHint: 'playback',
              sampleRate: audioInfo.sampleRate,
            });
            //* step5: send accept message *//
            ws.current?.send('accept');
          } else {
            console.error(`Invalid AudioInfo format: ${event.data}`);
            setError('invalid_audio_info');
          }
        }
      }
    };

    ws.current.onclose = (event: CloseEvent) => {
      console.log('WebSocket connection closed:', event.code, event.reason);
      setReadyState('disconnected');
    };

    ws.current.onerror = (error: Event) => {
      console.error('WebSocket error:', error);
      setError('websocket');
      setReadyState('disconnected');
    };
  }, []);

  // connect handler
  const connect = useCallback(() => {
    // check is WebSocket already connected or connecting
    if (ws.current && ws.current.readyState <= 1) {
      console.warn('WebSocket is already connected or connecting.');
      return;
    }

    console.log(`Connecting to ${url}...`);
    setReadyState('connecting');
    // reset error state
    setError(null);

    // create WebSocket instance
    ws.current = new WebSocket(url);
    setupEventListeners();
  }, [url, setupEventListeners]);

  // disconnect handler
  const disconnect = useCallback(() => {
    console.log('Disconnecting WebSocket...');
    ws.current?.close();
    audioContext.current?.close();
  }, []);

  // disconnect websocket when the component is unmounted.
  useEffect(() => {
    return () => {
      ws.current?.close();
      audioContext.current?.close();
    };
  }, []);

  return { audioInfoState, bpmState, readyState, error, connect, disconnect };
};

const App: FC = () => {
  // server URL state
  const [serverUrl, setServerUrl] = useState<string>('ws://localhost:7000');

  // useWebSocket hook
  const { audioInfoState, bpmState, readyState, error, connect, disconnect } = useWebSocket(serverUrl);

  // connect handler
  const handleConnect = () => {
    connect();
  };

  // disconnect handler
  const handleDisconnect = () => {
    disconnect();
  };

  // set server URL handler
  const handleServerUrlChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setServerUrl(e.target.value);
  };

  return (
    <main className="p-4 md:p-6 lg:p-8 grid grid-cols-1 md:grid-cols-3 gap-6">
      {/* server url setting form */}
      <div>
        <label htmlFor="serverUrl" className="block text-sm font-medium text-gray-600 mb-1">
          WebSocket URL
        </label>
        <input
          type="text"
          id="serverUrl"
          value={serverUrl}
          onChange={handleServerUrlChange}
          className="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition"
          disabled={readyState === 'connected' || readyState === 'connecting'}
        />
      </div>
      {/* connection control panel */}
      <div className="items-center justify-between">
        <div className={`px-3 py-1 text-sm font-semibold rounded-full`}>{readyState}</div>
        <div className={`px-3 py-1 text-sm font-semibold rounded-full`}>
          {audioInfoState
            ? `Channel: ${audioInfoState.channel}, Sample Rate: ${audioInfoState.sampleRate}, Bits Per Sample: ${audioInfoState.bitsPerSample}, PCM Format: ${audioInfoState.pcmFormat}`
            : 'No AudioInfo'}
        </div>
        <div className={`px-3 py-1 text-sm font-semibold rounded-full`}>
          BPM: {bpmState > 0 ? bpmState : 'Not Set'}
        </div>
        <div className="flex space-x-2">
          <button
            onClick={handleConnect}
            disabled={readyState === 'connected' || readyState === 'connecting'}
            className="px-4 py-2 bg-indigo-600 text-white font-semibold rounded-lg shadow-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-opacity-75 transition disabled:bg-gray-400 disabled:cursor-not-allowed"
          >
            接続
          </button>
          <button
            onClick={handleDisconnect}
            disabled={readyState !== 'connected'}
            className="px-4 py-2 bg-red-600 text-white font-semibold rounded-lg shadow-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-opacity-75 transition disabled:bg-gray-400 disabled:cursor-not-allowed"
          >
            切断
          </button>
        </div>
      </div>

      {/* error message */}
      {error && (
        <div className="bg-red-100 border-l-4 border-red-500 text-red-700 p-4 rounded-md" role="alert">
          <p className="font-bold">エラー</p>
          <p className="text-sm">接続に失敗しました。URLを確認するか、コンソールを参照してください。</p>
        </div>
      )}
    </main>
  );
};

export default App;
