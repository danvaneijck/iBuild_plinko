import React, { useEffect, useRef } from 'react';
import { Ball } from '../types/game';
import { ROWS_CONFIG } from '../config/multipliers';

interface PlinkoBoardProps {
  difficulty: 'easy' | 'medium' | 'hard';
  multipliers: number[];
  balls: Ball[];
}

export const PlinkoBoard: React.FC<PlinkoBoardProps> = ({ difficulty, multipliers, balls }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>();

  const rows = ROWS_CONFIG[difficulty];
  const pegRadius = 4;
  const ballRadius = 8;
  const spacing = 40;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;

    const drawPeg = (x: number, y: number) => {
      ctx.beginPath();
      ctx.arc(x, y, pegRadius, 0, Math.PI * 2);
      ctx.fillStyle = '#9E7FFF';
      ctx.fill();
      ctx.strokeStyle = '#7C3AED';
      ctx.lineWidth = 2;
      ctx.stroke();
    };

    const drawBall = (ball: Ball, progress: number) => {
      const startX = width / 2;
      const startY = 60;
      
      let currentX = startX;
      let currentY = startY;
      
      const completedRows = Math.floor(progress * ball.path.length);
      
      for (let i = 0; i < completedRows; i++) {
        currentY += spacing;
        currentX += ball.path[i] === 1 ? spacing / 2 : -spacing / 2;
      }
      
      if (completedRows < ball.path.length) {
        const rowProgress = (progress * ball.path.length) - completedRows;
        currentY += spacing * rowProgress;
        const direction = ball.path[completedRows] === 1 ? 1 : -1;
        currentX += (spacing / 2) * direction * rowProgress;
      }

      const gradient = ctx.createRadialGradient(currentX, currentY, 0, currentX, currentY, ballRadius);
      gradient.addColorStop(0, '#f472b6');
      gradient.addColorStop(1, '#ec4899');
      
      ctx.beginPath();
      ctx.arc(currentX, currentY, ballRadius, 0, Math.PI * 2);
      ctx.fillStyle = gradient;
      ctx.fill();
      ctx.strokeStyle = '#be185d';
      ctx.lineWidth = 2;
      ctx.stroke();
      
      ctx.shadowBlur = 15;
      ctx.shadowColor = '#f472b6';
      ctx.fill();
      ctx.shadowBlur = 0;
    };

    const animate = () => {
      ctx.clearRect(0, 0, width, height);

      // Draw pegs
      for (let row = 0; row < rows; row++) {
        const pegsInRow = row + 3;
        const rowY = 80 + row * spacing;
        const rowWidth = (pegsInRow - 1) * spacing;
        const startX = (width - rowWidth) / 2;

        for (let peg = 0; peg < pegsInRow; peg++) {
          const pegX = startX + peg * spacing;
          drawPeg(pegX, rowY);
        }
      }

      // Draw balls
      const now = Date.now();
      balls.forEach(ball => {
        const elapsed = now - parseInt(ball.id.split('-')[1]);
        const progress = Math.min(elapsed / 3000, 1);
        drawBall(ball, progress);
      });

      animationRef.current = requestAnimationFrame(animate);
    };

    animate();

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [balls, rows, difficulty]);

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
        width={800}
        height={rows * 40 + 160}
        className="w-full bg-gradient-to-b from-gray-900 to-gray-800 rounded-2xl border border-gray-700 shadow-2xl"
      />
      <div className="flex justify-center gap-1 mt-4">
        {multipliers.map((mult, idx) => (
          <div
            key={idx}
            className={`flex-1 max-w-[60px] py-3 rounded-lg bg-gradient-to-br ${getMultiplierColor(mult)} text-white font-bold text-center shadow-lg`}
          >
            {mult}x
          </div>
        ))}
      </div>
    </div>
  );
};
