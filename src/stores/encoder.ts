import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface EncodeJob {
  id: string;
  projectId: string;
  projectName: string;
  inputVideo: string;
  inputSub: string;
  outputPath: string;
  videoCodec: string;
  resolution: string;
  rateControl: string;
  crf: number;
  videoBitrate: string;
  preset: string;
  audioCodec: string;
  audioBitrate: string;
  subMode: string;
  format: string;
  durationSeconds: number;
  outputFps: string;
  
  status: 'pending' | 'encoding' | 'success' | 'error';
  percent: number;
  timeStr: string;
  fps: number;
}

export const useEncoderStore = defineStore('encoder', () => {
  const queue = ref<EncodeJob[]>([])
  const activeJobId = ref<string | null>(null)

  const addJob = (job: EncodeJob) => {
    queue.value.push(job)
  }

  const removeJob = (id: string) => {
    queue.value = queue.value.filter(j => j.id !== id)
  }

  const clearDone = () => {
    queue.value = queue.value.filter(j => j.status !== 'success' && j.status !== 'error')
  }

  const updateProgress = (id: string, percent: number, timeStr: string, fps: number, status: string) => {
    const job = queue.value.find(j => j.id === id)
    if (job) {
      job.percent = percent
      job.timeStr = timeStr
      job.fps = fps
      if (status === 'success' || status === 'error') {
        job.status = status as any
        if (activeJobId.value === id) activeJobId.value = null
      }
    }
  }

  return {
    queue,
    activeJobId,
    addJob,
    removeJob,
    clearDone,
    updateProgress
  }
})
