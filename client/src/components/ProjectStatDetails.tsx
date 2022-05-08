import {JSX} from 'solid-js';

const ProjectStatDetails: (props) => JSX.Element = (props) => {
    return (
        <div>
            <h4 class="text-5xl font-semibold text-center text-gray-800 dark:text-gray-200">Project details</h4>
            <p class="text-white text-center py-4">
                Here, you can see the progression of the unsafe usage within the project codebase, over time.
            </p>

            <div class="flex flex-col">
                <div class="overflow-x-auto sm:-mx-6 lg:-mx-8">
                    <div class="py-4 inline-block min-w-full sm:px-6 lg:px-8">
                        <div class="overflow-hidden">
                            <table class="min-w-full text-center">
                                <thead class="border-b bg-gray-800">
                                <tr>
                                    <th scope="col" class="text-white px-6 py-4">Name</th>
                                    <th scope="col" class="text-white px-6 py-4">Code lines</th>
                                    <th scope="col" class="text-white px-6 py-4">Unsafe lines</th>
                                    <th scope="col" class="text-white px-6 py-4">Created at</th>
                                    <th scope="col" class="text-white px-6 py-4">Updated at</th>
                                </tr>
                                </thead>
                                <Show when={props.data.length}>
                                    <tbody>
                                    <For each={props.data}>
                                        {
                                            (projectStat) => {
                                                return (
                                                    <tr>
                                                        <td class="font-medium text-white px-6 py-4">
                                                            <a
                                                                target="_blank"
                                                                href={projectStat.url}>{projectStat.name}
                                                            </a>
                                                        </td>
                                                        <td class="text-white px-6 py-4">{projectStat.code_lines}</td>
                                                        <td class="text-white px-6 py-4">{projectStat.unsafe_lines}</td>
                                                        <td class="text-white px-6 py-4">{projectStat.created_at}</td>
                                                        <td class="text-white px-6 py-4">{projectStat.updated_at}</td>
                                                    </tr>
                                                );
                                            }
                                        }
                                    </For>
                                    </tbody>
                                </Show>
                            </table>
                        </div>
                    </div>
                </div>
            </div>

            <div>
                <button
                    onclick={props.navigate}
                    type="button"
                    class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center inline-flex items-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
                    <svg class="w-6 h-6 dark:text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24"
                         xmlns="http://www.w3.org/2000/svg">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                              d="M10 19l-7-7m0 0l7-7m-7 7h18"></path>
                    </svg>
                    &nbsp;&nbsp;Back
                </button>
            </div>
        </div>
    );
};

export default ProjectStatDetails;
