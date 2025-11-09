import React, { useEffect, useRef, useState } from 'react';
import Matter from 'matter-js';
import { Ball, GameResult } from '../types/game';
import { ROWS_CONFIG } from '../config/multipliers';
import { useWindowWidth } from '../hooks/useWindowWidth';
import { AnimatePresence, motion } from 'framer-motion';

// --- Physics Constants ---
const PEG_RADIUS = 5;
const BALL_RADIUS = 9;
const SPACING = 45;
const BUCKET_HEIGHT = 40; // Height of the buckets at the bottom
const STEERING_FACTOR = 0.1; // Increased for stronger steering
const CANVAS_WIDTH = 800;

// --- Type extension for Matter.Body ---
// This lets us attach our game-specific data directly to the physics body
interface PlinkoBallBody extends Matter.Body {
  ballId: string;
  path: number[];
  currentRow: number;
  targetX?: number;

}

interface PlinkoBoardProps {
  difficulty: 'easy' | 'medium' | 'hard';
  multipliers: number[];
  balls: Ball[];
  gameHistory: GameResult[];
  onAnimationComplete: (ballId: string) => void;
}

// --- New Color Function for Canvas ---
const getMultiplierColorForCss = (multiplier: number) => {
  if (multiplier >= 100) return 'from-yellow-500 to-orange-500';
  if (multiplier >= 10) return 'from-green-500 to-emerald-500';
  if (multiplier >= 1) return 'from-blue-500 to-cyan-500';
  return 'from-red-500 to-pink-500';
};

// This function is for canvas drawing
const getMultiplierColorForCanvas = (multiplier: number) => {
  if (multiplier >= 100) return '#f59e0b'; // amber-500
  if (multiplier >= 10) return '#22c55e'; // green-500
  if (multiplier >= 1) return '#3b82f6'; // blue-500
  return '#ef4444'; // red-500
};

const getMultiplierTextColor = (multiplier: number) => {
  if (multiplier >= 100) return 'text-yellow-400';
  if (multiplier >= 10) return 'text-green-400';
  if (multiplier >= 1) return 'text-sky-400';
  return 'text-red-400';
};

const AUDIO_POOL_SIZE = 5; // The number of simultaneous sounds you want to support

export const PlinkoBoard: React.FC<PlinkoBoardProps> = ({
  difficulty,
  multipliers,
  gameHistory,
  balls,
  onAnimationComplete,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<Matter.Engine>();
  const renderRef = useRef<Matter.Render>();
  const runnerRef = useRef<Matter.Runner>();

  const pegsRef = useRef<Matter.Body[]>([]);

  const rows = ROWS_CONFIG[difficulty];
  const windowWidth = useWindowWidth();

  const historyContainerRef = useRef<HTMLDivElement>(null);

  const isMutedRef = useRef(true);
  const [isMuted, setIsMuted] = useState(true);

  const audioRef = useRef<HTMLAudioElement | null>(null);
  const audioPoolRef = useRef<HTMLAudioElement[]>([]);
  const currentAudioIndexRef = useRef(0);

  const toggleMute = () => {
    isMutedRef.current = !isMutedRef.current;
    setIsMuted(isMutedRef.current); // Update state for UI icon change
  };

  useEffect(() => {
    audioRef.current = new Audio('/plinko-[AudioTrimmer.com].mp3'); // Path relative to the public folder
  }, []);

  useEffect(() => {
    // Initialize the audio pool
    audioPoolRef.current = Array.from({ length: AUDIO_POOL_SIZE }, () => new Audio('/plinko-[AudioTrimmer.com].mp3'));
  }, []);

  useEffect(() => {
    if (historyContainerRef.current) {

      const timer = setTimeout(() => {
        historyContainerRef.current?.scrollTo({ top: -100, behavior: 'smooth' });
      }, 0);

      return () => clearTimeout(timer);
    }
  }, [gameHistory]); // Re-run whenever the history changes


  // --- Effect 1: One-Time Setup of the Physics World ---
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;


    const engine = Matter.Engine.create({ gravity: { y: 1 } });
    const render = Matter.Render.create({
      canvas: canvas,
      engine: engine,
      options: {
        width: 800,
        height: rows * SPACING + BUCKET_HEIGHT + (windowWidth < 500 ? 100 : 60),
        wireframes: false,
        background: 'transparent',
      },
    });
    const runner = Matter.Runner.create();

    engineRef.current = engine;
    renderRef.current = render;
    runnerRef.current = runner;

    const staticBodies: Matter.Body[] = [];
    pegsRef.current = [];

    // Pegs
    for (let row = 0; row < rows; row++) {
      const pegsInRow = row + 3;
      const rowY = 80 + row * SPACING;
      const rowWidth = (pegsInRow - 1) * SPACING;
      const startX = (800 - rowWidth) / 2;
      for (let i = 0; i < pegsInRow; i++) {
        const peg = Matter.Bodies.circle(startX + i * SPACING, rowY, PEG_RADIUS, {
          isStatic: true,
          label: `peg-${row}-${i}`,
          restitution: 0.5,
          render: { fillStyle: '#9E7FFF' },
        });
        staticBodies.push(peg);
        pegsRef.current.push(peg);
      }
    }

    // Boundaries and Buckets
    const groundY = rows * SPACING + BUCKET_HEIGHT + 40;
    staticBodies.push(Matter.Bodies.rectangle(400, groundY, 800, 20, { isStatic: true, render: { visible: false } }));
    staticBodies.push(Matter.Bodies.rectangle(-10, 300, 20, 1000, { isStatic: true, render: { visible: false } }));
    staticBodies.push(Matter.Bodies.rectangle(810, 300, 20, 1000, { isStatic: true, render: { visible: false } }));

    const bucketBottomY = rows * SPACING + 40 + (BUCKET_HEIGHT / 2);

    const bucketWidth = SPACING;
    const totalBucketsWidth = multipliers.length * bucketWidth;
    const bucketsStartX = (CANVAS_WIDTH - totalBucketsWidth) / 2;

    for (let i = 0; i <= multipliers.length; i++) {
      const x = bucketsStartX + i * bucketWidth;
      const bucketWall = Matter.Bodies.rectangle(x, bucketBottomY, 5, BUCKET_HEIGHT, {
        isStatic: true,
        render: { fillStyle: '#6b21a8' },
      });
      staticBodies.push(bucketWall);
    }


    Matter.Composite.add(engine.world, staticBodies);

    // --- EVENT: Draw Multipliers After Each Render Frame ---
    const drawMultipliers = () => {
      const ctx = render.context;
      if (!ctx) return;

      const bucketWidth = SPACING;
      const totalBucketsWidth = multipliers.length * bucketWidth;
      const bucketsStartX = (CANVAS_WIDTH - totalBucketsWidth) / 2;
      const textY = rows * SPACING + BUCKET_HEIGHT + 15; // Vertical position for the text

      ctx.font = 'bold 18px Arial';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';

      multipliers.forEach((mult, idx) => {
        const x = bucketsStartX + (idx * bucketWidth) + (bucketWidth / 2);
        ctx.fillStyle = getMultiplierColorForCanvas(mult);
        ctx.fillText(`${mult}x`, x, textY);
      });
    };

    Matter.Events.on(render, 'afterRender', drawMultipliers);


    const playHitSound = () => {
      if (!isMutedRef.current) {
        const audio = audioPoolRef.current[currentAudioIndexRef.current];
        if (audio) {
          audio.currentTime = 0;
          audio.play().catch(error => console.error("Audio play failed:", error));
        }
        // Move to the next audio element in the pool
        currentAudioIndexRef.current = (currentAudioIndexRef.current + 1) % AUDIO_POOL_SIZE;
      }
    };

    Matter.Events.on(engine, 'collisionStart', (event) => {
      for (const pair of event.pairs) {
        const { bodyA, bodyB } = pair;
        let ballBody: PlinkoBallBody | null = null;
        let pegBody: Matter.Body | null = null;

        if (bodyA.label === 'ball' && bodyB.label.startsWith('peg-')) {
          ballBody = bodyA as PlinkoBallBody;
          pegBody = bodyB;
        } else if (bodyB.label === 'ball' && bodyA.label.startsWith('peg-')) {
          ballBody = bodyB as PlinkoBallBody;
          pegBody = bodyA;
        }

        if (ballBody && pegBody) {
          playHitSound();
          const [_, pegRowStr, pegIndexStr] = pegBody.label.split('-');
          const pegRow = parseInt(pegRowStr);
          const pegIndex = parseInt(pegIndexStr);

          if (ballBody.currentRow < pegRow) {
            ballBody.currentRow = pegRow;
            const direction = ballBody.path[pegRow];

            if (direction === undefined) {
              ballBody.targetX = undefined;
              return;
            }


            if (pegRow === rows - 1) {
              const finalBucketIndex = ballBody.path.reduce((sum, dir) => sum + dir, 0);

              const bucketWidth = SPACING;
              const totalBucketsWidth = multipliers.length * bucketWidth;
              const bucketsStartX = (CANVAS_WIDTH - totalBucketsWidth) / 2;

              const finalTargetX = bucketsStartX + (finalBucketIndex * bucketWidth) + (bucketWidth / 2);

              ballBody.targetX = finalTargetX;

            } else {
              const nextPegIndex = pegIndex + direction;
              const nextRow = pegRow + 1;
              const targetPeg = pegsRef.current.find(p => p.label === `peg-${nextRow}-${nextPegIndex}`);

              if (targetPeg) {
                ballBody.targetX = targetPeg.position.x;
              }
            }
          }
        }
      }
    });

    Matter.Events.on(engine, 'beforeUpdate', () => {
      Matter.Composite.allBodies(engine.world).forEach(body => {
        if (body.label === 'ball') {
          const ballBody = body as PlinkoBallBody;
          if (ballBody.targetX !== undefined) {
            const error = ballBody.targetX - ballBody.position.x;
            const correctiveVx = error * STEERING_FACTOR;
            Matter.Body.setVelocity(ballBody, {
              x: correctiveVx,
              y: ballBody.velocity.y,
            });
          }
        }
      });
    });

    Matter.Runner.run(runner, engine);
    Matter.Render.run(render);

    return () => {
      // --- Cleanup ---
      Matter.Events.off(render, 'afterRender', drawMultipliers); // Remove the event listener
      Matter.Render.stop(render);
      Matter.Runner.stop(runner);
      Matter.Engine.clear(engine);
      render.canvas.getContext('2d')?.clearRect(0, 0, render.canvas.width, render.canvas.height);
    };
  }, [rows, multipliers, windowWidth]); // Add multipliers to dependency array


  // --- Effect 2: Dynamically Add and Remove Balls ---
  useEffect(() => {
    const engine = engineRef.current;
    if (!engine) return;

    balls.forEach((ball) => {
      const existingBody = Matter.Composite.allBodies(engine.world).find(b => (b as PlinkoBallBody).ballId === ball.id);
      if (!existingBody) {
        const ballBody = Matter.Bodies.circle(
          ball.x,
          ball.y,
          BALL_RADIUS,
          {
            label: 'ball',
            restitution: 0.6,
            render: { fillStyle: '#ec4899' },
          }
        ) as PlinkoBallBody;

        ballBody.ballId = ball.id;
        ballBody.path = ball.path;
        ballBody.currentRow = -1;
        ballBody.targetX = undefined;

        Matter.Composite.add(engine.world, ballBody);
      }
    });

    Matter.Composite.allBodies(engine.world).forEach((body) => {
      if (body.label === 'ball') {
        const ballBody = body as PlinkoBallBody;
        const stillExistsInProps = balls.some(b => b.id === ballBody.ballId);
        if (!stillExistsInProps) {
          Matter.Composite.remove(engine.world, body);
        }
      }
    });

  }, [balls]);


  // --- Effect 3: Handle Animation Completion ---
  useEffect(() => {
    const engine = engineRef.current;
    if (!engine) return;
    const checkBallsInterval = setInterval(() => {
      Matter.Composite.allBodies(engine.world).forEach(body => {
        if (body.label === 'ball') {
          if (body.position.y > rows * SPACING + 40) {
            const ballBody = body as PlinkoBallBody;
            onAnimationComplete(ballBody.ballId);
          }
        }
      });
    }, 1000);
    return () => clearInterval(checkBallsInterval);
  }, [rows, onAnimationComplete]);

  return (
    <div className="relative">
      <canvas
        ref={canvasRef}
        key={rows}
        className="w-full bg-gradient-to-b from-gray-900 to-gray-800 rounded-2xl border border-gray-700 shadow-2xl"
      />

      {/* --- 3. UI ELEMENT: VOLUME BUTTON --- */}
      <div className="absolute top-4 right-4">
        <button
          onClick={toggleMute}
          className="w-10 h-10 flex items-center justify-center bg-slate-900/70 backdrop-blur-sm rounded-full text-white/70 hover:text-white transition-colors border border-white/10"
          aria-label={isMuted ? 'Unmute' : 'Mute'}
        >
          {isMuted ? (
            // Muted Icon (Speaker with X)
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5">
              <path strokeLinecap="round" strokeLinejoin="round" d="M17.25 9.75L19.5 12m0 0l2.25 2.25M19.5 12l2.25-2.25M19.5 12l-2.25 2.25m-10.5-6l4.72-4.72a.75.75 0 011.28.53v15.88a.75.75 0 01-1.28.53l-4.72-4.72H4.51c-.88 0-1.704-.507-1.938-1.354A9.01 9.01 0 012.25 12c0-.83.112-1.633.322-2.396C2.806 8.756 3.63 8.25 4.51 8.25H6.75z" />
            </svg>
          ) : (
            // Unmuted Icon (Speaker with waves)
            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" className="w-5 h-5">
              <path strokeLinecap="round" strokeLinejoin="round" d="M19.114 5.636a9 9 0 010 12.728M16.463 8.288a5.25 5.25 0 010 7.424M6.75 8.25l4.72-4.72a.75.75 0 011.28.53v15.88a.75.75 0 01-1.28.53l-4.72-4.72H4.51c-.88 0-1.704-.507-1.938-1.354A9.01 9.01 0 012.25 12c0-.83.112-1.633.322-2.396C2.806 8.756 3.63 8.25 4.51 8.25H6.75z" />
            </svg>
          )}
        </button>
      </div>

      {/* --- UPDATED: Game History Display with Animations --- */}
      {/* <div
        ref={historyContainerRef} // Attach the ref here
        className="absolute top-4 left-4 flex flex-col-reverse gap-2 max-h-[300px] overflow-y-auto pr-2 scrollbar-hide"
      >
        <AnimatePresence>
          {gameHistory.sort((a, b) => (b.timestamp - a.timestamp) || ((a.eventIndex ?? 0) - (b.eventIndex ?? 0)))
            .slice(-10).reverse().map((game, index) => (
              <motion.div
                key={game.ballId || game.timestamp + index}
                layout
                initial={{ opacity: 0, y: -20, scale: 0.8 }}
                animate={{ opacity: 1, y: 0, scale: 1 }}
                exit={{ opacity: 0, scale: 0.5, transition: { duration: 0.2 } }}
                className={`
                w-14 px-3 py-1 text-xs font-bold text-center rounded-full shadow-lg
                bg-slate-900/70 backdrop-blur-sm
                border border-white/10
                origin-bottom
                ${getMultiplierTextColor(game.multiplier)}
              `}
              >
                {game.multiplier}
              </motion.div>
            ))}
        </AnimatePresence>
      </div> */}
    </div>
  );
};