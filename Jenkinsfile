
properties([
  parameters([
    string(
      name: 'Build_Image',
      defaultValue: 'All',
      description: 'Build image in the jenkins'
    )
  ])
])


node('ccis') {
    try{
        stage('Setup Environment') {
            cleanWs()
            checkout scm
            def image = docker.build('cureos-builder', '.')
            env.IMAGE_ID = image.id
        }

        stage('Build with Make') {
            docker.image(env.IMAGE_ID).inside("--user jenkins") {
                sh '''
                    echo "=== Environment Check ==="
                    rustc --version
                    cargo --version
                    make --version

                    echo ""
                    echo "=== Building with Make ==="
                    make all
                '''
            }
        }

        stage('Archive') {
            archiveArtifacts artifacts: 'build/**/*.img, bin/**/*',
                            fingerprint: true
        }

    } finally  {
       stage('Cleanup') {
           sh """
               docker ps -a --filter "label=jenkins-build=${JOB_NAME}" -q | xargs -r docker rm -f || true
               docker rmi ${env.IMAGE_ID} || true
           """
       }
       stage('Notify to Discord') {
           withCredentials([string(credentialsId: '725eb3cc-b38a-412a-b83c-cd5a5017f8b8', variable: 'https://discord.com/api/webhooks/1438181389534236793/pdbdWbJL1MAAw_YCtXjZ8YqjJ38VQkf_HA5QzQruKEF94aFaTGDcRciZVsauIgHJJJdw')]) {
              script {
                    def status = currentBuild.currentResult
                    def desc = """
                    **Job:** ${env.JOB_NAME}
                    **Build #:** ${env.BUILD_NUMBER}
                    **Status:** ${status}
                    **URL:** ${env.BUILD_URL}
                    """.stripIndent()

                    discordSend(
                        webhookURL: DISCORD_WEBHOOK_URL,
                        title: "Jenkins Build - ${status}",
                        description: desc,
                        result: status
                    )
              }
           }
       }
    }
}