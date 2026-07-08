import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Project {
  id: string;
  name: string;
  project_type: string;
  original_language: string;
  target_language: string;
  season: number | null;
  episode: number | null;
  output_folder: string;
  source_video: string;
  thumbnail: string;
  last_modified: string;
}

export const useProjectStore = defineStore('project', () => {
  const currentProject = ref<Project | null>(null)
  const recentProjects = ref<Project[]>([])

  const setProject = (project: Project) => {
    currentProject.value = project
  }

  const fetchProjects = async () => {
    try {
      const projects: Project[] = await invoke('get_recent_projects')
      recentProjects.value = projects
    } catch (e) {
      console.error("Failed to fetch projects:", e)
    }
  }

  const createNewProject = async (data: Partial<Project>) => {
    try {
      const newProjectData = {
        id: "",
        name: data.name || "Untitled Project",
        project_type: data.project_type || "Other",
        original_language: data.original_language || "English",
        target_language: data.target_language || "Indonesian",
        season: data.season || null,
        episode: data.episode || null,
        output_folder: data.output_folder || "documents/subpilot",
        source_video: data.source_video || "",
        thumbnail: data.thumbnail || "",
        last_modified: ""
      }
      
      const newProject: Project = await invoke('create_project', { project: newProjectData })
      recentProjects.value.unshift(newProject)
      return newProject
    } catch (e) {
      console.error("Failed to create project:", e)
      throw e
    }
  }

  const updateProject = async (projectData: Project) => {
    try {
      const updatedProject: Project = await invoke('update_project', { project: projectData })
      
      const index = recentProjects.value.findIndex(p => p.id === updatedProject.id)
      if (index !== -1) {
        recentProjects.value[index] = updatedProject
      }
      
      if (currentProject.value?.id === updatedProject.id) {
        currentProject.value = updatedProject
      }
      
      return updatedProject
    } catch (e) {
      console.error("Failed to update project:", e)
      throw e
    }
  }

  const removeProject = async (id: string) => {
    try {
      await invoke('delete_project', { id })
      recentProjects.value = recentProjects.value.filter(p => p.id !== id)
      if (currentProject.value?.id === id) {
        currentProject.value = null
      }
    } catch (e) {
      console.error("Failed to delete project:", e)
      throw e
    }
  }

  const copyProject = async (id: string) => {
    try {
      const copiedProject: Project = await invoke('duplicate_project', { id })
      recentProjects.value.unshift(copiedProject)
    } catch (e) {
      console.error("Failed to copy project:", e)
      throw e
    }
  }

  return { 
    currentProject, 
    recentProjects, 
    setProject, 
    fetchProjects, 
    createNewProject, 
    updateProject,
    removeProject, 
    copyProject 
  }
})
