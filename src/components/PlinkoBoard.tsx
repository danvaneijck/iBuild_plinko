import React, { useEffect, useRef } from 'react';
import Matter from 'matter-js';
import { Ball } from '../types/game';
import { ROWS_CONFIG } from '../config/multipliers';
import { useWindowWidth } from '../hooks/useWindowWidth';

// --- Physics Constants ---
const PEG_RADIUS = 5;
const BALL_RADIUS = 9;
const SPACING = 45;
const BUCKET_HEIGHT = 40; // Height of the buckets at the bottom
const STEERING_FACTOR = 0.15; // Increased for stronger steering
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
  onAnimationComplete: (ballId: string) => void;
}

export const PlinkoBoard: React.FC<PlinkoBoardProps> = ({
  difficulty,
  multipliers,
  balls,
  onAnimationComplete,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const engineRef = useRef<Matter.Engine>();
  const renderRef = useRef<Matter.Render>();
  const runnerRef = useRef<Matter.Runner>();

  // Storing pegs in a ref allows us to access them in the event handlers
  // without needing them in the useEffect dependency array.
  const pegsRef = useRef<Matter.Body[]>([]);

  const rows = ROWS_CONFIG[difficulty];
  const windowWidth = useWindowWidth();


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
    pegsRef.current = []; // Clear previous pegs

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
        pegsRef.current.push(peg); // Store peg reference
      }
    }

    // Boundaries and Buckets (using the "knife-edge" fix from before)
    const groundY = rows * SPACING + BUCKET_HEIGHT + 40;
    staticBodies.push(Matter.Bodies.rectangle(400, groundY, 800, 20, { isStatic: true, render: { visible: false } }));
    staticBodies.push(Matter.Bodies.rectangle(-10, 300, 20, 1000, { isStatic: true, render: { visible: false } }));
    staticBodies.push(Matter.Bodies.rectangle(810, 300, 20, 1000, { isStatic: true, render: { visible: false } }));

    // const bucketBottomY = rows * SPACING + 60 + (BUCKET_HEIGHT / 2);
    // const bucketTopY = rows * SPACING + 60 - (BUCKET_HEIGHT / 2);

    // // --- Corrected Bucket Alignment Logic ---
    // // A bucket's width should be equal to the horizontal distance between pegs.
    // const bucketWidth = SPACING;

    // // The total width occupied by all buckets.
    // const totalBucketsWidth = multipliers.length * bucketWidth;

    // // Calculate the starting X-position to center the entire block of buckets in the canvas.
    // const bucketsStartX = (CANVAS_WIDTH - totalBucketsWidth) / 2;

    // // Create (multipliers.length + 1) walls to form (multipliers.length) buckets.
    // for (let i = 0; i <= multipliers.length; i++) {
    //   const x = bucketsStartX + i * bucketWidth;
    //   const bucketWall = Matter.Bodies.rectangle(x, bucketBottomY, 5, BUCKET_HEIGHT, {
    //     isStatic: true,
    //     render: { fillStyle: '#6b21a8' },
    //   });
    //   const wallTopper = Matter.Bodies.circle(x, bucketTopY, 2, {
    //     isStatic: true,
    //     render: { fillStyle: '#6b21a8' },
    //   });
    //   staticBodies.push(bucketWall);
    // }

    Matter.Composite.add(engine.world, staticBodies);

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

    // --- EVENT 2: Steer the Ball Every Frame ---
    Matter.Events.on(engine, 'beforeUpdate', () => {
      Matter.Composite.allBodies(engine.world).forEach(body => {
        if (body.label === 'ball') {
          const ballBody = body as PlinkoBallBody;
          // If the ball has a target, steer it
          if (ballBody.targetX !== undefined) {
            // Calculate the difference between where the ball is and where it should be
            const error = ballBody.targetX - ballBody.position.x;

            // Apply a corrective velocity.
            const correctiveVx = error * STEERING_FACTOR;

            // Set the velocity, preserving the vertical speed from gravity
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
      Matter.Render.stop(render);
      Matter.Runner.stop(runner);
      Matter.Engine.clear(engine);
      render.canvas.getContext('2d')?.clearRect(0, 0, render.canvas.width, render.canvas.height);
    };
  }, [rows, multipliers.length, windowWidth]);


  // --- Effect 2: Dynamically Add and Remove Balls ---
  useEffect(() => {
    const engine = engineRef.current;
    if (!engine) return;

    balls.forEach((ball) => {
      const existingBody = Matter.Composite.allBodies(engine.world).find(b => (b as PlinkoBallBody).ballId === ball.id);
      if (!existingBody) {
        const ballBody = Matter.Bodies.circle(
          ball.x, // Use the exact starting X
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
        ballBody.currentRow = -1; // Start before the first row
        ballBody.targetX = undefined; // No initial target

        Matter.Composite.add(engine.world, ballBody);
      }
    });

    // Remove old balls (logic remains the same)
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


  // --- Effect 3: Handle Animation Completion (logic remains the same) ---
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

  const getMultiplierColor = (multiplier: number) => {
    if (multiplier >= 100) return 'from-yellow-500 to-orange-500';
    if (multiplier >= 10) return 'from-green-500 to-emerald-500';
    if (multiplier >= 1) return 'from-blue-500 to-cyan-500';
    return 'from-red-500 to-pink-500';
  };

  return (
    <div className="relative">
      <canvas
        ref={canvasRef}
        // Set the key to force a re-mount when rows change, ensuring a clean slate
        key={rows}
        className="w-full bg-gradient-to-b from-gray-900 to-gray-800 rounded-2xl border border-gray-700 shadow-2xl"
      />
      <div
        className="absolute bottom-0 left-0 right-0 flex justify-center items-end "
        // You can adjust this padding to move the labels up or down as needed
        style={{ paddingBottom: '0px' }}
      >
        {multipliers.map((mult, idx) => (
          <div
            key={idx}
            style={{ width: `40px` }}
            className={`py-3 text-xs md:text-sm  rounded-t-lg bg-gradient-to-br ${getMultiplierColor(
              mult
            )} text-white font-bold text-center shadow-lg text-ellipsis overflow-hidden`}
          >
            {mult}x
          </div>
        ))}
      </div>
    </div>
  );
};